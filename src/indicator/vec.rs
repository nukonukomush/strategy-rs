use crate::time::*;
use crate::*;

pub struct VecIndicator<G, V> {
    granularity: G,
    offset: Time<G>,
    vec: Vec<V>,
}

impl<G, V> VecIndicator<G, V>
where
    G: Granularity + Copy,
{
    pub fn new(offset: Time<G>, source: Vec<V>) -> Self {
        Self {
            granularity: offset.granularity(),
            offset: offset,
            vec: source,
        }
    }
}

impl<G, V> Indicator<G, V> for VecIndicator<G, V>
where
    V: Clone,
    G: Granularity + Copy,
{
    fn granularity(&self) -> G {
        self.granularity
    }
}

impl<G, V> FuncIndicator<G, V> for VecIndicator<G, V>
where
    V: Clone,
    G: Granularity + Copy,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V> {
        let i = (time.timestamp() - self.offset.timestamp()) / self.granularity.unit_duration();
        if i >= 0 && i < (self.vec.len() as i64) {
            MaybeValue::Value(self.vec[i as usize].clone())
        } else {
            MaybeValue::OutOfRange
        }
    }
}

use std::collections::HashMap;
pub struct HashMapIndicator<G, V> {
    map: HashMap<Time<G>, V>,
    granularity: G,
}

impl<G, V> HashMapIndicator<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    pub fn new(granularity: G) -> Self {
        Self::from_map(HashMap::new(), granularity)
    }
    pub fn from_map(map: HashMap<Time<G>, V>, granularity: G) -> Self {
        Self {
            map: map,
            granularity: granularity,
        }
    }
    pub fn insert(&mut self, time: Time<G>, value: V) {
        self.map.insert(time, value);
    }
}

impl<G, V> Indicator<G, Option<V>> for HashMapIndicator<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    fn granularity(&self) -> G {
        self.granularity
    }
}

impl<G, V> FuncIndicator<G, Option<V>> for HashMapIndicator<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    fn value(&self, time: Time<G>) -> MaybeValue<Option<V>> {
        // TODO: out of range
        match self.map.get(&time) {
            Some(v) => MaybeValue::Value(Some(v.clone())),
            None => MaybeValue::Value(None),
        }
    }
}

pub fn from_vec<V, G>(offset: Time<G>, vec: Vec<V>) -> HashMapIndicator<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    let mut h = HashMap::new();
    vec.into_iter().enumerate().for_each(|(i, v)| {
        h.insert(offset + (i as i64), v);
    });
    HashMapIndicator {
        map: h,
        granularity: offset.granularity(),
    }
}

// TODO: feature
#[cfg(ffi)]
mod ffi {
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
        b_ptr: *mut Rc<RefCell<VecIndicator<VarGranularity, V>>>,
        t_ptr: *mut IndicatorPtr<V>,
    }

    macro_rules! define_vec_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                offset: CTime,
                array: *const $t,
                length: c_int,
            ) -> Ptr<$t> {
                let array: &[$t] = std::slice::from_raw_parts(array, length as usize);
                let ptr = Rc::new(RefCell::new(VecIndicator::new(
                    offset.into(),
                    array.to_vec(),
                )));
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
        };
    }

    define_vec_methods!(f64, vec_new_f64, vec_destroy_f64);
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
        b_ptr: *mut Rc<RefCell<HashMapIndicator<VarGranularity, V>>>,
        t_ptr: *mut IndicatorPtr<V>,
    }

    macro_rules! define_hash_methods {
        ($t:ty, $new:ident, $destroy:ident, $set:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(granularity: VarGranularity) -> Ptr<$t> {
                let ptr = Rc::new(RefCell::new(HashMapIndicator::new(granularity)));
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
        let ptr = Rc::new(RefCell::new(HashMapIndicator::new(granularity)));
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
    fn test_vec() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Value(1.0), Value(2.0), Value(3.0), Value(4.0), Value(5.0)];

        let vec = VecIndicator::new(offset, source.clone());
        let result = (0..5).map(|i| vec.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_hash() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Value(Some(1.0)),
            Value(Some(2.0)),
            Value(Some(3.0)),
            Value(Some(4.0)),
            Value(Some(5.0)),
        ];

        let hash = from_vec(offset, source.clone());
        let result = (0..5).map(|i| hash.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
