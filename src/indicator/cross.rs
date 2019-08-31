use super::*;
use crate::indicator::ordering::*;

pub struct Cross<G, I> {
    source: I,
    phantom: std::marker::PhantomData<G>,
}

impl<G, I> Cross<G, I> {
    pub fn from_ord(source: I) -> Self {
        Self {
            source: source,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, V, I1, I2> Cross<G, Ordering<G, V, I1, I2>>
where
    G: Granularity,
    V: PartialOrd,
    I1: Indicator<G, V>,
    I2: Indicator<G, V>,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        let source = Ordering::new(source_1, source_2);
        Cross::from_ord(source)
    }
}

impl<G, I> Indicator<G, CrossState> for Cross<G, I>
where
    G: Granularity + Copy,
    I: Indicator<G, std::cmp::Ordering>,
{
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

impl<G, I> FuncIndicator<G, CrossState> for Cross<G, I>
where
    G: Granularity + Copy,
    I: FuncIndicator<G, std::cmp::Ordering>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<CrossState> {
        use std::cmp::Ordering::*;
        use CrossState::*;

        // TODO: refactor using cmpl
        let current_ord = try_value!(self.source.value(time));
        if current_ord != Equal {
            let mut i = time - 1;
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

// #[cfg(ffi)]
pub mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;
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

    define_cross_methods!(f64, cross_new_f64, cross_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;

    #[test]
    fn test_cross() {
        use CrossState::GtToLt as gtl;
        use CrossState::LtToGt as ltg;
        use CrossState::NotCrossed as not;

        let offset = Time::<S5>::new(0, S5);
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
