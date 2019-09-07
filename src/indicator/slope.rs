use super::*;
use crate::seq::*;

pub struct Slope<S, V, I> {
    source: I,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V>,
}

impl<S, V, I> Slope<S, V, I>
where
    I: Indicator<S, V>,
    V: std::ops::Sub,
{
    pub fn new(source: I) -> Self {
        Self {
            source: source,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

impl<S, V, I> Indicator<S, V::Output> for Slope<S, V, I>
where
    S: Sequence,
    I: Indicator<S, V>,
    V: std::ops::Sub,
{
}

impl<S, V, I> FuncIndicator<S, V::Output> for Slope<S, V, I>
where
    S: Sequence,
    I: FuncIndicator<S, V>,
    V: std::ops::Sub,
{
    fn value(&self, seq: S) -> MaybeValue<V::Output> {
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

    type IPtr<S, V> = Ptr<S, V, Slope<S, V, FuncIndicatorPtr<S, V>>>;

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
