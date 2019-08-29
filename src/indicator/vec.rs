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

// #[cfg(ffi)]
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

    type IPtr<V> = Ptr<V, VecIndicator<VarGranularity, V>>;

    macro_rules! define_vec_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                offset: CTime,
                array: *const $t,
                length: c_int,
            ) -> IPtr<$t> {
                let array: &[$t] = std::slice::from_raw_parts(array, length as usize);
                let ptr = VecIndicator::new(offset.into(), array.to_vec()).into_sync_ptr();
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                    i_ptr: ptr::null_mut(),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: IPtr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.f_ptr);
                destroy(ptr.i_ptr);
            }
        };
    }

    define_vec_methods!(f64, vec_new_f64, vec_destroy_f64);
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
}
