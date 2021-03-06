use crate::seq::*;
use crate::*;
use std::collections::HashMap;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Storage<S, V> {
    begin: S,
    end: S,
    map: HashMap<S, V>,
}

impl<S, V> Storage<S, V>
where
    S: Sequence,
{
    pub fn new(offset: S) -> Self {
        // Self::from_map(HashMap::new(), granularity)
        Self {
            begin: offset,
            end: offset,
            map: HashMap::new(),
        }
    }

    // pub fn from_map(map: HashMap<S, V>) -> Self {
    //     Self {
    //         map: map,
    //     }
    // }

    pub fn add(&mut self, seq: S, value: V) {
        debug_assert!(seq >= self.end);
        self.map.insert(seq, value);
        self.end = seq + 1;
    }

    pub fn from_vec(offset: S, vec: Vec<V>) -> Self {
        let len = vec.len();
        if len == 0 {
            Self::new(offset)
        } else {
            let mut h = HashMap::new();
            vec.into_iter().enumerate().for_each(|(i, v)| {
                h.insert(offset + (i as i64), v);
            });
            Self {
                begin: offset,
                end: offset + len as i64,
                map: h,
            }
        }
    }
}

impl<S, V> Indicator for Storage<S, V>
where
    S: Sequence,
    V: std::fmt::Debug,
{
    type Seq = S;
    type Val = Option<V>;
}

impl<S, V> FuncIndicator for Storage<S, V>
where
    S: Sequence,
    V: Clone + std::fmt::Debug,
{
    fn value(&self, seq: S) -> MaybeValue<Option<V>> {
        if seq < self.begin {
            Fixed(OutOfRange)
        } else if self.end <= seq {
            NotFixed
        } else {
            let v = match self.map.get(&seq) {
                Some(v) => Some(v.clone()),
                None => None,
            };
            Fixed(InRange(v))
        }
    }
}

#[cfg(feature = "ffi")]
mod hash_ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, Option<V>, Storage<S, V>>;

    pub unsafe fn new<S, CS, V>(offset: CS) -> IPtr<S, V>
    where
        S: Sequence + 'static,
        CS: Into<S>,
        V: Clone + std::fmt::Debug + 'static,
    {
        let ptr = Rc::new(RefCell::new(Storage::new(offset.into())));
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
        }
    }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(offset: $cs) -> IPtr<$s, $v> {
                new(offset)
            }
        };
    }

    pub unsafe fn add<S, CS, V, CV>(ptr: IPtr<S, V>, seq: CS, value: CV)
    where
        S: Sequence,
        CS: Into<S>,
        V: Clone,
        CV: Into<V> + Clone,
    {
        let ptr = ptr.b_ptr;
        if ptr.is_null() {
            return;
        }

        let ptr = &*ptr;
        ptr.borrow_mut().add(seq.into(), value.into());
    }

    macro_rules! define_add {
        ($ptr:ty, $cs:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(ptr: $ptr, seq: $cs, value: $cv) {
                add(ptr, seq, value)
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, storage_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, storage_new_tid_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, storage_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, storage_destroy_tid_f64);

    define_add!(IPtr<GTime<Var>, f64>, CTime, f64, storage_add_time_f64);
    define_add!(IPtr<TransactionId, f64>, i64, f64, storage_add_tid_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::time::*;

    #[test]
    fn test_from_vec() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Fixed(InRange(Some(1.0))),
            Fixed(InRange(Some(2.0))),
            Fixed(InRange(Some(3.0))),
            Fixed(InRange(Some(4.0))),
            Fixed(InRange(Some(5.0))),
        ];

        let storage = Storage::from_vec(offset, source.clone());
        let result = (0..5)
            .map(|i| storage.value(offset + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_add_ok() {
        let offset = Time::<S5>::new(0);
        let expect = vec![
            Fixed(InRange(Some(1.0))),
            Fixed(InRange(Some(2.0))),
            Fixed(InRange(None)),
            Fixed(InRange(Some(3.0))),
            NotFixed,
        ];

        let mut storage = Storage::new(offset);
        storage.add(offset + 0, 1.0);
        storage.add(offset + 1, 2.0);
        storage.add(offset + 3, 3.0);

        let result = (0..5)
            .map(|i| storage.value(offset + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    #[should_panic]
    fn test_add_ng_1() {
        let offset = Time::<S5>::new(0);
        let mut storage = Storage::new(offset);
        storage.add(offset + 0, 1.0);
        storage.add(offset + 3, 3.0);
        storage.add(offset + 1, 2.0);
    }

    #[test]
    #[should_panic]
    fn test_add_ng_2() {
        let offset = Time::<S5>::new(0);
        let mut storage = Storage::new(offset);
        storage.add(offset + 0, 1.0);
        storage.add(offset + 3, 3.0);
        storage.add(offset + 3, 2.0);
    }
}
