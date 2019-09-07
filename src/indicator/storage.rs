use crate::time::*;
use crate::seq::*;
use crate::*;
use std::collections::HashMap;

pub struct Storage<S, V> {
    begin: S,
    end: S,
    map: HashMap<S, V>,
}

impl<S, V> Storage<S, V>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
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

impl<S, V> Indicator<S, Option<V>> for Storage<S, V>
where
    // V: Clone,
    // S: Granularity + Eq + std::hash::Hash + Copy,
    S: Sequence,
{
    // fn granularity(&self) -> S {
    //     self.begin.granularity()
    // }
}

impl<S, V> FuncIndicator<S, Option<V>> for Storage<S, V>
where
    S: Sequence,
    V: Clone,
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
{
    fn value(&self, seq: S) -> MaybeValue<Option<V>> {
        if self.begin <= seq && seq < self.end {
            match self.map.get(&seq) {
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
    use crate::seq::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    type IPtr<V> = Ptr<Option<V>, Storage<VarGranularity, V>>;

    macro_rules! define_storage_methods {
        ($t:ty, $new:ident, $destroy:ident, $add:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(offset: CTime) -> IPtr<$t> {
                let ptr = Rc::new(RefCell::new(Storage::new(offset.into())));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: IPtr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.f_ptr);
            }

            #[no_mangle]
            pub unsafe extern "C" fn $add(ptr: IPtr<$t>, seq: CTime, value: $t) {
                let ptr = ptr.b_ptr;
                if ptr.is_null() {
                    return;
                }

                let ptr = &*ptr;
                ptr.borrow_mut().add(seq.into(), value);
            }
        };
    }

    define_storage_methods!(f64, storage_new_f64, storage_destroy_f64, storage_add_f64);

    // use crate::position::ffi::*;
    // use crate::position::*;
    // #[no_mangle]
    // pub unsafe extern "C" fn hash_new_simpleposition(
    //     granularity: VarGranularity,
    // ) -> Ptr<SimplePosition> {
    //     let ptr = Rc::new(RefCell::new(Storage::new(granularity)));
    //     Ptr {
    //         b_ptr: Box::into_raw(Box::new(ptr.clone())),
    //         t_ptr: Box::into_raw(Box::new(IndicatorPtr(ptr))),
    //     }
    // }

    // #[no_mangle]
    // pub unsafe extern "C" fn hash_destroy_simpleposition(ptr: Ptr<SimplePosition>) {
    //     destroy(ptr.b_ptr);
    //     destroy(ptr.t_ptr);
    // }

    // #[no_mangle]
    // pub unsafe extern "C" fn hash_set_simpleposition(
    //     ptr: Ptr<SimplePosition>,
    //     seq: CTime,
    //     value: CSimplePosition,
    // ) {
    //     let ptr = ptr.b_ptr;
    //     if ptr.is_null() {
    //         return;
    //     }

    //     let ptr = &*ptr;
    //     ptr.borrow_mut().insert(seq.into(), value.into());
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MaybeValue::*;
    use crate::granularity::*;

    #[test]
    fn test_from_vec() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Value(Some(1.0)),
            Value(Some(2.0)),
            Value(Some(3.0)),
            Value(Some(4.0)),
            Value(Some(5.0)),
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
