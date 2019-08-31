use super::*;

pub struct Slope<G, V, I> {
    source: I,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V>,
}

impl<G, V, I> Slope<G, V, I>
where
    I: Indicator<G, V>,
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

impl<G, V, I> Indicator<G, V::Output> for Slope<G, V, I>
where
    G: Granularity + Copy,
    I: Indicator<G, V>,
    V: std::ops::Sub,
{
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

impl<G, V, I> FuncIndicator<G, V::Output> for Slope<G, V, I>
where
    G: Granularity + Copy,
    I: FuncIndicator<G, V>,
    V: std::ops::Sub,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V::Output> {
        let cur = try_value!(self.source.value(time));
        let prev = try_value!(self.source.value(time - 1));
        MaybeValue::Value(cur - prev)
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

    define_slope_methods!(f64, slope_new_f64, slope_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;

    #[test]
    fn test_slope() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 4.0, 8.0, 6.0];
        let expect = vec![OutOfRange, Value(1.0), Value(2.0), Value(4.0), Value(-2.0)];
        let source = VecIndicator::new(offset, source);
        let slope = Slope::new(source);

        let result = (0..5).map(|i| slope.value(offset + i)).collect::<Vec<_>>();

        assert_eq!(result, expect);
    }
}
