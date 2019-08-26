use crate::time::*;
use crate::*;

pub struct VecIndicator<G, V> {
    granularity: G,
    offset: Time<G>,
    vec: Vec<V>,
}

impl<G, V> Indicator<G, V> for VecIndicator<G, V>
where
    V: Clone,
    G: Granularity + Copy,
{
    fn value(&self, time: Time<G>) -> Option<V> {
        let i = (time.timestamp() - self.offset.timestamp()) / self.granularity.unit_duration();
        if i >= 0 && i < (self.vec.len() as i64) {
            Some(self.vec[i as usize].clone())
        } else {
            None
        }
    }
    fn granularity(&self) -> G {
        self.granularity
    }
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

use std::collections::HashMap;
pub struct HashMapIndicator<G, V> {
    map: HashMap<Time<G>, V>,
    granularity: G,
}

impl<G, V> Indicator<G, V> for HashMapIndicator<G, V>
where
    V: Clone,
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    fn value(&self, time: Time<G>) -> Option<V> {
        match self.map.get(&time) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    fn granularity(&self) -> G {
        self.granularity
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];

        let vec = VecIndicator::new(offset, source.clone());
        let result = (0..5).map(|i| vec.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_hash() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];

        let hash = from_vec(offset, source.clone());
        let result = (0..5).map(|i| hash.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
