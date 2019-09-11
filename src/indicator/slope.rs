use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Slope<I> {
    source: I,
}

impl<I> Slope<I> {
    pub fn new(source: I) -> Self {
        Self { source: source }
    }
}

impl<V1, V2, I> Indicator for Slope<I>
where
    V1: std::ops::Sub<Output = V2> + std::fmt::Debug,
    V2: std::fmt::Debug,
    I: Indicator<Val = V1>,
{
    type Seq = I::Seq;
    type Val = V1::Output;
}

impl<V1, V2, I> FuncIndicator for Slope<I>
where
    V1: std::ops::Sub<Output = V2> + std::fmt::Debug,
    V2: std::fmt::Debug,
    I: FuncIndicator<Val = V1>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let cur = try_value!(self.source.value(seq));
        let prev = try_value!(self.source.value(seq - 1));
        Fixed(InRange(cur - prev))
    }
}

#[cfg(ffi)]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, Slope<FuncIndicatorPtr<S, V>>>;

    pub unsafe fn new<S, V>(source: *mut FuncIndicatorPtr<S, V>) -> IPtr<S, V>
    where
        S: Sequence + 'static,
        V: Clone + std::ops::Sub<Output = V> + 'static,
    {
        let source = (*source).clone();
        let ptr = Slope::new(source).into_sync_ptr();
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
        }
    }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(source: *mut FuncIndicatorPtr<$s, $v>) -> IPtr<$s, $v> {
                new(source)
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, slope_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, slope_new_tid_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, slope_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, slope_destroy_tid_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_slope() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 4.0, 8.0, 6.0];
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(InRange(1.0)),
            Fixed(InRange(2.0)),
            Fixed(InRange(4.0)),
            Fixed(InRange(-2.0)),
        ];
        let source = VecIndicator::new(offset, source);
        let slope = Slope::new(source);

        let result = (0..5).map(|i| slope.value(offset + i)).collect::<Vec<_>>();

        assert_eq!(result, expect);
    }
}
