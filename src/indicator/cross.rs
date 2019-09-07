use super::*;
use crate::seq::*;
use crate::indicator::ordering::*;

pub struct Cross<S, I> {
    source: I,
    phantom: std::marker::PhantomData<S>,
}

impl<S, I> Cross<S, I> {
    pub fn from_ord(source: I) -> Self {
        Self {
            source: source,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<S, V, I1, I2> Cross<S, Ordering<S, V, I1, I2>>
where
    S: Sequence,
    V: PartialOrd,
    I1: Indicator<S, V>,
    I2: Indicator<S, V>,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        let source = Ordering::new(source_1, source_2);
        Cross::from_ord(source)
    }
}

impl<S, I> Indicator<S, CrossState> for Cross<S, I>
where
    S: Sequence,
    I: Indicator<S, std::cmp::Ordering>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, I> FuncIndicator<S, CrossState> for Cross<S, I>
where
    S: Sequence,
    I: FuncIndicator<S, std::cmp::Ordering>,
{
    fn value(&self, seq: S) -> MaybeValue<CrossState> {
        use std::cmp::Ordering::*;
        use CrossState::*;

        // TODO: refactor using cmpl
        let current_ord = try_value!(self.source.value(seq));
        if current_ord != Equal {
            let mut i = seq - 1;
            while let MaybeValue::Value(past_ord) = self.source.value(i) {
                match (past_ord, current_ord) {
                    (Greater, Less) => return MaybeValue::Value(GtToLt),
                    (Less, Greater) => return MaybeValue::Value(LtToGt),
                    (Greater, Greater) => return MaybeValue::Value(NotCrossed),
                    (Less, Less) => return MaybeValue::Value(NotCrossed),
                    _ => (),
                }
                i = i - 1;
            }
        }
        MaybeValue::Value(NotCrossed)
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
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::seq::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

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

    type IPtr<V> = Ptr<
        CrossState,
        Cross<
            VarGranularity,
            Ordering<VarGranularity, V, FuncIndicatorPtr<V>, FuncIndicatorPtr<V>>,
        >,
    >;

    macro_rules! define_cross_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                source_1: *mut FuncIndicatorPtr<$t>,
                source_2: *mut FuncIndicatorPtr<$t>,
            ) -> IPtr<$t> {
                let source_1 = (*source_1).clone();
                let source_2 = (*source_2).clone();
                let ptr = Rc::new(RefCell::new(Cross::new(source_1, source_2)));
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

    define_cross_methods!(f64, cross_new_f64, cross_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;
    use crate::granularity::*;

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
            .map(|v| Value(v))
            .collect::<Vec<_>>();
        let source_1 = VecIndicator::new(offset, source_1);
        let source_2 = VecIndicator::new(offset, source_2);
        let cross = Cross::new(source_1, source_2);

        let result = (0..10).map(|i| cross.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }
}
