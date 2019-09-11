use super::*;
use crate::indicator::ordering::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Cross<I> {
    source: I,
}

impl<I> Cross<I> {
    pub fn from_ord(source: I) -> Self {
        Self { source: source }
    }
}

impl<I1, I2> Cross<Ordering<I1, I2>>
where
    I1: Indicator,
    I2: Indicator,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        let source = Ordering::new(source_1, source_2);
        Cross::from_ord(source)
    }
}

impl<I> Indicator for Cross<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = CrossState;
}

impl<I> FuncIndicator for Cross<I>
where
    I: FuncIndicator<Val = std::cmp::Ordering>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        use std::cmp::Ordering::*;
        use CrossState::*;

        // TODO: refactor using cmpl
        let current_ord = try_value!(self.source.value(seq));
        if current_ord != Equal {
            let mut i = seq - 1;
            while let Fixed(InRange(past_ord)) = self.source.value(i) {
                match (past_ord, current_ord) {
                    (Greater, Less) => return Fixed(InRange(GtToLt)),
                    (Less, Greater) => return Fixed(InRange(LtToGt)),
                    (Greater, Greater) => return Fixed(InRange(NotCrossed)),
                    (Less, Less) => return Fixed(InRange(NotCrossed)),
                    _ => (),
                }
                i = i - 1;
            }
        }
        Fixed(InRange(NotCrossed))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrossState {
    NotCrossed,
    LtToGt,
    GtToLt,
}

#[cfg(ffi)]
pub mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::time::ffi::*;

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum CCrossState {
        NotCrossed = 0,
        LtToGt = 1,
        GtToLt = -1,
    }
    impl Default for CCrossState {
        fn default() -> Self {
            CCrossState::NotCrossed
        }
    }

    impl From<CrossState> for CCrossState {
        fn from(s: CrossState) -> Self {
            match s {
                CrossState::NotCrossed => CCrossState::NotCrossed,
                CrossState::LtToGt => CCrossState::LtToGt,
                CrossState::GtToLt => CCrossState::GtToLt,
            }
        }
    }

    type IPtr<S, V> =
        Ptr<S, CrossState, Cross<Ordering<FuncIndicatorPtr<S, V>, FuncIndicatorPtr<S, V>>>>;

    pub unsafe fn new<S, V>(
        source_1: *mut FuncIndicatorPtr<S, V>,
        source_2: *mut FuncIndicatorPtr<S, V>,
    ) -> IPtr<S, V>
    where
        S: Sequence + 'static,
        V: Clone + PartialOrd + 'static,
    {
        let source_1 = (*source_1).clone();
        let source_2 = (*source_2).clone();
        let ptr = Cross::new(source_1, source_2).into_sync_ptr();
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
        }
    }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                source_1: *mut FuncIndicatorPtr<$s, $v>,
                source_2: *mut FuncIndicatorPtr<$s, $v>,
            ) -> IPtr<$s, $v> {
                new(source_1, source_2)
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, cross_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, cross_new_tid_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, cross_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, cross_destroy_tid_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_cross() {
        use CrossState::GtToLt as gtl;
        use CrossState::LtToGt as ltg;
        use CrossState::NotCrossed as not;

        let offset = Time::<S5>::new(0);
        let source_1 = vec![0.0, 0.0, 2.0, 2.0, 0.0, 1.0, 1.0, 2.0, 1.0, 0.0_f64];
        let source_2 = vec![1.0; 10];
        let expected = vec![not, not, ltg, not, gtl, not, not, ltg, not, gtl]
            .into_iter()
            .map(|v| Fixed(InRange(v)))
            .collect::<Vec<_>>();
        let source_1 = VecIndicator::new(offset, source_1);
        let source_2 = VecIndicator::new(offset, source_2);
        let cross = Cross::new(source_1, source_2);

        let result = (0..10).map(|i| cross.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }
}
