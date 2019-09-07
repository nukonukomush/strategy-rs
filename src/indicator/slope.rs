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
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
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

#[cfg(ffi)]
mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::seq::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    type IPtr<V> = Ptr<V, Slope<VarGranularity, V, FuncIndicatorPtr<V>>>;

    macro_rules! define_slope_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(source: *mut FuncIndicatorPtr<$t>) -> IPtr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(Slope::new(source)));
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
        };
    }

    define_slope_methods!(f64, slope_new_f64, slope_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;
    use crate::granularity::*;

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
