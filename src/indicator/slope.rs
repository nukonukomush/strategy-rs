use super::*;

pub struct Slope<I> {
    source: I,
}

impl<I> Slope<I> {
    pub fn new(source: I) -> Self {
        Self { source: source }
    }
}

impl<V, I> Indicator for Slope<I>
where
    V: std::ops::Sub,
    I: Indicator<Val = V>,
{
    type Seq = I::Seq;
    type Val = V::Output;
}

impl<V, I> FuncIndicator for Slope<I>
where
    V: std::ops::Sub,
    I: FuncIndicator<Val = V>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let cur = try_value!(self.source.value(seq));
        let prev = try_value!(self.source.value(seq - 1));
        MaybeValue::Value(cur - prev)
    }
}

// #[cfg(ffi)]
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

    // macro_rules! define_slope_methods {
    //     ($t:ty, $new:ident, $destroy:ident) => {
    //         #[no_mangle]
    //         pub unsafe extern "C" fn $new(source: *mut FuncIndicatorPtr<$t>) -> IPtr<$t> {
    //             let source = (*source).clone();
    //             let ptr = Rc::new(RefCell::new(Slope::new(source)));
    //             Ptr {
    //                 b_ptr: Box::into_raw(Box::new(ptr.clone())),
    //                 f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
    //             }
    //         }

    //         #[no_mangle]
    //         pub unsafe extern "C" fn $destroy(ptr: IPtr<$t>) {
    //             destroy(ptr.b_ptr);
    //             destroy(ptr.f_ptr);
    //         }
    //     };
    // }

    // define_slope_methods!(f64, slope_new_f64, slope_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;
    use MaybeValue::*;

    #[test]
    fn test_slope() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 4.0, 8.0, 6.0];
        let expect = vec![OutOfRange, Value(1.0), Value(2.0), Value(4.0), Value(-2.0)];
        let source = VecIndicator::new(offset, source);
        let slope = Slope::new(source);

        let result = (0..5).map(|i| slope.value(offset + i)).collect::<Vec<_>>();

        assert_eq!(result, expect);
    }
}
