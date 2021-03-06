use crate::seq::*;
use crate::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct VecIndicator<S, V> {
    offset: S,
    vec: Vec<V>,
}

impl<S, V> VecIndicator<S, V> {
    pub fn new(offset: S, source: Vec<V>) -> Self {
        Self {
            offset: offset,
            vec: source,
        }
    }

    pub fn add(&mut self, value: V) {
        self.vec.push(value)
    }
}

impl<S, V> Indicator for VecIndicator<S, V>
where
    S: Sequence,
    V: std::fmt::Debug,
{
    type Seq = S;
    type Val = V;
}

impl<S, V> FuncIndicator for VecIndicator<S, V>
where
    V: Clone + std::fmt::Debug,
    S: Sequence,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let i = seq.distance_from(&self.offset);
        if i < 0 {
            Fixed(OutOfRange)
        } else if (self.vec.len() as i64) <= i {
            NotFixed
        } else {
            Fixed(InRange(self.vec[i as usize].clone()))
        }
    }
}

// impl<S, V> FuncIndicator for VecIndicator<S, V>
// where
//     V: Clone + std::fmt::Debug,
//     S: Sequence,
// {
// }

#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, VecIndicator<S, V>>;

    pub unsafe fn new<S, CS, V, CV>(offset: CS, array: *const CV, length: c_int) -> IPtr<S, V>
    where
        S: Sequence + 'static,
        CS: Into<S>,
        V: Clone + std::fmt::Debug + 'static,
        CV: Into<V> + Clone,
    {
        let array: &[CV] = std::slice::from_raw_parts(array, length as usize);
        let array = array.iter().map(|cv| cv.clone().into()).collect::<Vec<_>>();
        let ptr = VecIndicator::new(offset.into(), array).into_sync_ptr();
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
        }
    }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                offset: $cs,
                array: *const $cv,
                length: c_int,
            ) -> IPtr<$s, $v> {
                new(offset, array, length)
            }
        };
    }

    pub unsafe fn add<S, V, CV>(ptr: IPtr<S, V>, value: CV)
    where
        V: Clone,
        CV: Into<V> + Clone,
    {
        let ptr = ptr.b_ptr;
        if ptr.is_null() {
            return;
        }

        let ptr = &*ptr;
        ptr.borrow_mut().add(value.into());
    }

    macro_rules! define_add {
        ($ptr:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(ptr: $ptr, value: $cv) {
                add(ptr, value)
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, vec_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, vec_new_tid_f64);
    define_new!(TickId, i64, f64, f64, vec_new_tick_id_f64);
    define_new!(TickId, i64, GTime<Var>, CTime, vec_new_tick_id_time);

    define_destroy!(IPtr<GTime<Var>, f64>, vec_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, vec_destroy_tid_f64);
    define_destroy!(IPtr<TickId, f64>, vec_destroy_tick_id_f64);
    define_destroy!(IPtr<TickId, GTime<Var>>, vec_destroy_tick_id_time);

    define_add!(IPtr<GTime<Var>, f64>, f64, vec_add_time_f64);
    define_add!(IPtr<TransactionId, f64>, f64, vec_add_tid_f64);
    define_add!(IPtr<TickId, f64>, f64, vec_add_tick_id_f64);
    define_add!(IPtr<TickId, GTime<Var>>, CTime, vec_add_tick_id_time);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::time::*;

    #[test]
    fn test_vec() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Fixed(InRange(1.0)),
            Fixed(InRange(2.0)),
            Fixed(InRange(3.0)),
            Fixed(InRange(4.0)),
            Fixed(InRange(5.0)),
        ];

        let vec = VecIndicator::new(offset, source.clone());
        let result = (0..5).map(|i| vec.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
