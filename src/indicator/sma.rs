use super::*;
use crate::seq::*;
use crate::time::*;
use crate::*;

pub struct Sma<S, I> {
    source: I,
    period: isize,
    phantom: std::marker::PhantomData<S>,
}

impl<S, I> Sma<S, I> {
    pub fn new(source: I, period: usize) -> Self {
        Self {
            source: source,
            period: period as isize,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<S, I> Indicator<S, f64> for Sma<S, I>
where
    S: Sequence,
    I: Indicator<S, f64>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, I> FuncIndicator<S, f64> for Sma<S, I>
where
    S: Sequence,
    I: FuncIndicator<S, f64>,
{
    fn value(&self, seq: S) -> MaybeValue<f64> {
        let mut sum = 0.0;
        let begin = seq + 1 - (self.period as i64);
        // for i in (begin..=seq).rev() {
        //     let v = self.source.value(i)?;
        //     sum += v;
        // }
        let mut tmp = seq;
        while tmp >= begin {
            let v = try_value!(self.source.value(tmp));
            sum += v;
            tmp = tmp - 1;
        }
        MaybeValue::Value(sum / self.period as f64)
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

    type IPtr<V> = Ptr<V, Sma<VarGranularity, FuncIndicatorPtr<V>>>;

    macro_rules! define_sma_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(source: *mut FuncIndicatorPtr<$t>, period: c_int) -> IPtr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(Sma::new(source, period as usize)));
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

    define_sma_methods!(f64, sma_new_f64, sma_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use MaybeValue::*;
    use crate::indicator::cached::*;
    use crate::vec::*;
    use crate::granularity::*;

    #[test]
    fn test_sma() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![OutOfRange, OutOfRange, Value(2.0), Value(3.0), Value(4.0)];
        // let sma_pre = Sma::new(source, 3);
        // let sma = Cached::new(sma_pre);
        let sma_pre = Sma::new(VecIndicator::new(offset, source.clone()), 3);
        let sma = LRUCache::new(10, sma_pre);

        let result = (0..5).map(|i| sma.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
