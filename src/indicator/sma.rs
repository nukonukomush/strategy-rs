use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Sma<I> {
    source: I,
    period: isize,
}

impl<I> Sma<I> {
    pub fn new(source: I, period: usize) -> Self {
        Self {
            source: source,
            period: period as isize,
        }
    }
}

impl<I> Indicator for Sma<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

macro_rules! calc_sma {
    ($v:expr, $seq:ident, $self:ident) => {{
        let begin = $seq + 1 - ($self.period as i64);
        let mut sum = try_value!($v);
        let mut tmp = $seq - 1;
        while tmp >= begin {
            let v = try_value!($self.source.value(tmp));
            sum += v;
            tmp = tmp - 1;
        }
        Fixed(InRange(sum / $self.period as f64))
    }};
}

impl<I> FuncIndicator for Sma<I>
where
    I: FuncIndicator<Val = f64>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        calc_sma!(self.source.value(seq), seq, self)
        // let begin = seq + 1 - (self.period as i64);
        // let mut sum = try_value!(self.source.value(seq));
        // let mut tmp = seq - 1;
        // while tmp >= begin {
        //     let v = try_value!(self.source.value(tmp));
        //     sum += v;
        //     tmp = tmp - 1;
        // }
        // Fixed(InRange(sum / self.period as f64))
    }
}

impl<I> Provisional for Sma<I>
where
    I: FuncIndicator<Val = f64> + Provisional<Val = f64>,
{
    fn provisional_value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        calc_sma!(self.source.provisional_value(seq), seq, self)
    }
}

#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, Sma<FuncIndicatorPtr<S, V>>>;

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
        let sma_pre = Sma::new(VecIndicator::new(offset, source.clone()), 3);
        let sma = LRUCache::new(10, sma_pre);

        let result = (0..5).map(|i| sma.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_sma_p() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(OutOfRange),
            Fixed(InRange(2.0)),
            Fixed(InRange(3.0)),
            Fixed(InRange(4.0)),
        ];
        let source = ProvisionalExt::new(VecIndicator::new(offset, source)).into_sync_ptr();
        let sma = Sma::new(source.clone(), 3);

        let result = (0..5).map(|i| sma.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);

        source.borrow_mut().set_provisional_value(9.0);
        assert_eq!(sma.value(offset + 5), NotFixed);
        assert_eq!(sma.provisional_value(offset + 5), Fixed(InRange(6.0)));
    }
}
