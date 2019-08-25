use crate::time::*;
use crate::*;

pub struct VecIndicator<G, V> {
    offset: Time<G>,
    vec: Vec<V>,
}

impl<G, V> Indicator<G, V> for VecIndicator<G, V>
where
    V: Clone,
    G: Granularity,
{
    fn value(&self, time: Time<G>) -> Option<V> {
        let i = (time.timestamp() - self.offset.timestamp()) / G::unit_duration();
        if i >= 0 && i < (self.vec.len() as i64) {
            Some(self.vec[i as usize].clone())
        } else {
            None
        }
    }
}

impl<G, V> VecIndicator<G, V> {
    pub fn new(offset: Time<G>, source: Vec<V>) -> Self {
        Self {
            offset: offset,
            vec: source,
        }
    }
}

use std::collections::HashMap;

impl<G, V> Indicator<G, V> for HashMap<Time<G>, V>
where
    V: Clone,
    G: Granularity,
{
    fn value(&self, time: Time<G>) -> Option<V> {
        match self.get(&time) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}

pub fn from_vec<V, G>(offset: Time<G>, vec: Vec<V>) -> HashMap<Time<G>, V>
where
    V: Clone,
    G: Granularity,
{
    let mut h = HashMap::new();
    vec.into_iter().enumerate().for_each(|(i, v)| {
        h.insert(offset + (i as i64), v);
    });
    h
}

// use std::mem::drop;
// use std::os::raw::*;
// use std::ptr;

// use std::cell::RefCell;
// use std::rc::Rc;

// macro_rules! define_vec_methods {
//     ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
//         #[no_mangle]
//         pub unsafe extern "C" fn $new(
//             array: *const $t,
//             length: c_int,
//         ) -> *mut Rc<RefCell<Vec<$t>>> {
//             let array: &[$t] = std::slice::from_raw_parts(array, length as usize);
//             let obj = Box::new(Rc::new(RefCell::new(array.to_vec())));
//             Box::into_raw(obj)
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $trait(
//             obj: *mut Rc<RefCell<Vec<$t>>>,
//         ) -> *mut IndicatorPtr<$t> {
//             if obj.is_null() {
//                 return ptr::null_mut();
//             }
//             Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $destroy(obj: *mut Rc<RefCell<Vec<$t>>>) {
//             if obj.is_null() {
//                 return;
//             }
//             // ここ Box にする必要ある？？
//             let boxed = Box::from_raw(obj);
//             drop(boxed);
//         }
//     };
// }

// define_vec_methods!(f64, vec_new_f64, vec_trait_f64, vec_destroy_f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];

        let vec = VecIndicator::new(offset, source.clone());
        let result = (0..5).map(|i| vec.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_hash() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];

        let hash = from_vec(offset, source.clone());
        let result = (0..5).map(|i| hash.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
