use super::*;
use crate::indicator::rolling::*;
use MaybeFixed::*;
use MaybeInRange::*;

// pub struct Sma<I>(Rolling<I, fn(FixedSizeWindow<I::Seq, I>) -> MaybeValue<I::Val>>)
// where
//     I: Indicator;

// impl<I> Sma<I>
// where
//     I: FuncIndicator<Val = f64>,
// {
//     pub fn new(source: I, period: usize) -> Self {
//         Sma(source.rolling(period, |w| w.mean()))
//     }
// }

// impl<I> Indicator for Sma<I>
// where
//     I: Indicator,
// {
//     type Seq = I::Seq;
//     type Val = I::Val;
// }

// impl<I> FuncIndicator for Sma<I>
// where
//     I: FuncIndicator<Val = f64>,
// {
//     #[inline]
//     fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
//         self.0.value(seq)
//     }
// }

pub fn sma<I>(
    source: I,
    period: usize,
) -> Rolling<I, fn(FixedSizeWindow<I::Seq, I>) -> MaybeValue<I::Val>>
where
    I: FuncIndicator<Val = f64>,
{
    source.rolling(period, |w| w.mean())
}

#[allow(non_camel_case_types)]
pub struct Sma_2<I> {
    source: I,
    period: isize,
}

impl<I> Sma_2<I> {
    pub fn new(source: I, period: usize) -> Self {
        Self {
            source: source,
            period: period as isize,
        }
    }
}

impl<I> Indicator for Sma_2<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I> FuncIndicator for Sma_2<I>
where
    I: FuncIndicator<Val = f64>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let begin = seq + 1 - (self.period as i64);
        let mut sum = try_value!(self.source.value(seq));
        let mut tmp = seq - 1;
        while tmp >= begin {
            let v = try_value!(self.source.value(tmp));
            sum += v;
            tmp = tmp - 1;
        }
        Fixed(InRange(sum / self.period as f64))
    }
}

#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::time::ffi::*;

    type I<S, V> = FuncIndicatorPtr<S, V>;
    type IPtr<S, V> = Ptr<S, V, Rolling<I<S, V>, fn(FixedSizeWindow<S, I<S, V>>) -> MaybeValue<V>>>;
    // pub struct Sma<I>(Rolling<I, fn(FixedSizeWindow<I::Seq, I>) -> MaybeValue<I::Val>>)

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
                let ptr = Rc::new(RefCell::new(sma(source, period as usize)));
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

    #[test]
    fn test_sma() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(OutOfRange),
            Fixed(InRange(2.0)),
            Fixed(InRange(3.0)),
            Fixed(InRange(4.0)),
        ];
        // let sma_pre = Sma::new(source, 3);
        // let sma = Cached::new(sma_pre);
        let sma_pre = sma(VecIndicator::new(offset, source.clone()), 3);
        let sma = LRUCache::new(10, sma_pre);

        let result = (0..5).map(|i| sma.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
