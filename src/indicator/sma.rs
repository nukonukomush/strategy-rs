use super::*;

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

impl<S, V, I> Indicator<S, V> for Sma<S, I>
where
    S: Sequence,
    I: Indicator<S, V>,
{
}

impl<S, I> FuncIndicator<S, f64> for Sma<S, I>
where
    S: Sequence,
    I: FuncIndicator<S, f64>,
{
    fn value(&self, seq: S) -> MaybeValue<f64> {
        let mut sum = 0.0;
        let begin = seq + 1 - (self.period as i64);
        let mut tmp = seq;
        while tmp >= begin {
            let v = try_value!(self.source.value(tmp));
            sum += v;
            tmp = tmp - 1;
        }
        MaybeValue::Value(sum / self.period as f64)
    }
}
// impl<S, I> Indicator<S, f64> for Sma<S, I>
// where
//     S: Sequence,
//     I: Indicator<S, f64>,
// {
// }

// impl<S, I> FuncIndicator<S, f64> for Sma<S, I>
// where
//     S: Sequence,
//     I: FuncIndicator<S, f64>,
// {
//     fn value(&self, seq: S) -> MaybeValue<f64> {
//         let mut sum = 0.0;
//         let begin = seq + 1 - (self.period as i64);
//         let mut tmp = seq;
//         while tmp >= begin {
//             let v = try_value!(self.source.value(tmp));
//             sum += v;
//             tmp = tmp - 1;
//         }
//         MaybeValue::Value(sum / self.period as f64)
//     }
// }

// #[cfg(ffi)]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, Sma<S, FuncIndicatorPtr<S, V>>>;

    // pub unsafe fn new<S, CS, V, CV>(
    //     source: *mut FuncIndicatorPtr<S, V>,
    //     period: c_int,
    // ) -> IPtr<S, V>
    // where
    //     S: Sequence + 'static,
    //     CS: Into<S>,
    //     V: 'static,
    //     CV: Into<V>,
    // {
    //     let source = (*source).clone();
    //     let ptr = Rc::new(RefCell::new(Sma::new(source, period as usize)));
    //     Ptr {
    //         b_ptr: Box::into_raw(Box::new(ptr.clone())),
    //         f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
    //     }
    // }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                source: *mut FuncIndicatorPtr<$s, $v>,
                period: c_int,
            ) -> IPtr<$s, $v> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(Sma::new(source, period as usize)));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, sma_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, sma_new_tid_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, sma_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, sma_destroy_tid_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::indicator::cached::*;
    use crate::vec::*;
    use MaybeValue::*;

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
