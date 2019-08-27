use crate::time::*;
use crate::*;
use std::collections::HashMap;

pub struct Storage<G, V> {
    begin: Time<G>,
    end: Time<G>,
    map: HashMap<Time<G>, V>,
}

impl<G, V> Storage<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy + Ord,
{
    pub fn new(offset: Time<G>) -> Self {
        // Self::from_map(HashMap::new(), granularity)
        Self {
            begin: offset,
            end: offset,
            map: HashMap::new(),
        }
    }

    // pub fn from_map(map: HashMap<Time<G>, V>) -> Self {
    //     Self {
    //         map: map,
    //     }
    // }

    pub fn add(&mut self, time: Time<G>, value: V) {
        debug_assert!(time >= self.end);
        self.map.insert(time, value);
        self.end = time + 1;
    }

    pub fn from_vec(offset: Time<G>, vec: Vec<V>) -> Self
    where
        V: Clone,
        G: Granularity + Eq + std::hash::Hash + Copy,
    {
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

impl<G, V> Indicator<G, Option<V>> for Storage<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    fn granularity(&self) -> G {
        self.begin.granularity()
    }
}

impl<G, V> FuncIndicator<G, Option<V>> for Storage<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy + Ord,
{
    fn value(&self, time: Time<G>) -> MaybeValue<Option<V>> {
        if self.begin <= time && time < self.end {
            match self.map.get(&time) {
                Some(v) => MaybeValue::Value(Some(v.clone())),
                None => MaybeValue::Value(None),
            }
        } else {
            MaybeValue::OutOfRange
        }
    }
}

#[cfg(ffi)]
mod hash_ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    #[repr(C)]
    pub struct Ptr<V> {
        b_ptr: *mut Rc<RefCell<Storage<VarGranularity, V>>>,
        t_ptr: *mut IndicatorPtr<V>,
    }

    macro_rules! define_hash_methods {
        ($t:ty, $new:ident, $destroy:ident, $set:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(granularity: VarGranularity) -> Ptr<$t> {
                let ptr = Rc::new(RefCell::new(Storage::new(granularity)));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    t_ptr: Box::into_raw(Box::new(IndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: Ptr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.t_ptr);
            }

            #[no_mangle]
            pub unsafe extern "C" fn $set(ptr: Ptr<$t>, time: CTime, value: $t) {
                let ptr = ptr.b_ptr;
                if ptr.is_null() {
                    return;
                }

                let ptr = &*ptr;
                ptr.borrow_mut().insert(time.into(), value);
            }
        };
    }

    define_hash_methods!(f64, hash_new_f64, hash_destroy_f64, hash_set_f64);

    use crate::position::ffi::*;
    use crate::position::*;
    #[no_mangle]
    pub unsafe extern "C" fn hash_new_simpleposition(
        granularity: VarGranularity,
    ) -> Ptr<SimplePosition> {
        let ptr = Rc::new(RefCell::new(Storage::new(granularity)));
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            t_ptr: Box::into_raw(Box::new(IndicatorPtr(ptr))),
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn hash_destroy_simpleposition(ptr: Ptr<SimplePosition>) {
        destroy(ptr.b_ptr);
        destroy(ptr.t_ptr);
    }

    #[no_mangle]
    pub unsafe extern "C" fn hash_set_simpleposition(
        ptr: Ptr<SimplePosition>,
        time: CTime,
        value: CSimplePosition,
    ) {
        let ptr = ptr.b_ptr;
        if ptr.is_null() {
            return;
        }

        let ptr = &*ptr;
        ptr.borrow_mut().insert(time.into(), value.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MaybeValue::*;

    #[test]
    fn test_from_vec() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Value(Some(1.0)),
            Value(Some(2.0)),
            Value(Some(3.0)),
            Value(Some(4.0)),
            Value(Some(5.0)),
        ];

        let storage = Storage::from_vec(offset, source.clone());
        let result = (0..5).map(|i| storage.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_add_ok() {
        let offset = Time::new(0, S5);
        let expect = vec![
            Value(Some(1.0)),
            Value(Some(2.0)),
            Value(None),
            Value(Some(3.0)),
            OutOfRange,
        ];

        let mut storage = Storage::new(offset);
        storage.add(offset + 0, 1.0);
        storage.add(offset + 1, 2.0);
        storage.add(offset + 3, 3.0);

        let result = (0..5).map(|i| storage.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    #[should_panic]
    fn test_add_ng() {
        let offset = Time::new(0, S5);
        let mut storage = Storage::new(offset);
        storage.add(offset + 0, 1.0);
        storage.add(offset + 3, 3.0);
        storage.add(offset + 1, 2.0);
    }
}
