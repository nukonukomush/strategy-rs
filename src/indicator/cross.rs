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
    fn value(&self, time: Time<G>) -> Option<CrossState> {
        use std::cmp::Ordering::*;
        use CrossState::*;

        // TODO: refactor
        if let Some(current_ord) = self.source.value(time) {
            // if index != 0 && current_ord != Equal {
            //     for i in (0..=(index - 1)).rev() {
            //         if let Some(past_ord) = self.source.value(i) {
            //             match (past_ord, current_ord) {
            //                 (Greater, Less) => return Some(GtToLt),
            //                 (Less, Greater) => return Some(LtToGt),
            //                 (Greater, Greater) => return Some(NotCrossed),
            //                 (Less, Less) => return Some(NotCrossed),
            //                 _ => (),
            //             }
            //         }
            //     }
            // }
            if current_ord != Equal {
                let mut i = time - 1;
                while let Some(past_ord) = self.source.value(i) {
                    match (past_ord, current_ord) {
                        (Greater, Less) => return Some(GtToLt),
                        (Less, Greater) => return Some(LtToGt),
                        (Greater, Greater) => return Some(NotCrossed),
                        (Less, Less) => return Some(NotCrossed),
                        _ => (),
                    }
                    i = i - 1;
                }
            }
            Some(NotCrossed)
        } else {
            None
        }
    }
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrossState {
    NotCrossed,
    LtToGt,
    GtToLt,
}

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

    #[repr(C)]
    pub struct Ptr<V> {
        b_ptr: *mut Rc<
            RefCell<
                Cross<
                    VarGranularity,
                    Ordering<VarGranularity, V, IndicatorPtr<V>, IndicatorPtr<V>>,
                >,
            >,
        >,
        t_ptr: *mut IndicatorPtr<CrossState>,
    }

    macro_rules! define_cross_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                source_1: *mut IndicatorPtr<$t>,
                source_2: *mut IndicatorPtr<$t>,
            ) -> Ptr<$t> {
                let source_1 = (*source_1).clone();
                let source_2 = (*source_2).clone();
                let ptr = Rc::new(RefCell::new(Cross::new(source_1, source_2)));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    t_ptr: Box::into_raw(Box::new(IndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: Ptr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.t_ptr);
            }
        };
    }

    define_cross_methods!(f64, cross_new_f64, cross_destroy_f64);
    // #[no_mangle]
    // pub unsafe extern "C" fn cross_new_f64(
    //     source_1: *mut IndicatorPtr<f64>,
    //     source_2: *mut IndicatorPtr<f64>,
    // ) -> *mut CrossPtr<f64> {
    //     let source_1 = (*source_1).clone();
    //     let source_2 = (*source_2).clone();
    //     let cross = Rc::new(RefCell::new(cross(source_1, source_2)));
    //     Box::into_raw(Box::new(cross))
    // }

    // #[no_mangle]
    // pub unsafe extern "C" fn cross_trait_f64(obj: *mut CrossPtr<f64>) -> *mut IndicatorPtr<CrossState> {
    //     if obj.is_null() {
    //         return ptr::null_mut();
    //     }
    //     Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
    // }

    // #[no_mangle]
    // pub unsafe extern "C" fn cross_destroy_f64(obj: *mut CrossPtr<f64>) {
    //     if obj.is_null() {
    //         return;
    //     }
    //     let boxed = Box::from_raw(obj);
    //     drop(boxed);
    // }

    // // #[no_mangle]
    // // pub unsafe extern "C" fn cross_trait_ordering(
    // //     obj: *mut Rc<RefCell<Cross<IndicatorPtr<OrderingValue>>>>,
    // // ) -> *mut IndicatorPtr<CrossState> {
    // //     if obj.is_null() {
    // //         return ptr::null_mut();
    // //     }
    // //     Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
    // // }

    // // #[no_mangle]
    // // pub unsafe extern "C" fn cross_destroy_ordering(
    // //     obj: *mut Rc<RefCell<Cross<IndicatorPtr<OrderingValue>>>>,
    // // ) {
    // //     if obj.is_null() {
    // //         return;
    // //     }
    // //     let boxed = Box::from_raw(obj);
    // //     drop(boxed);
    // // }

    // // macro_rules! define_cross_methods {
    // //     ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
    // //         #[no_mangle]
    // //         pub unsafe extern "C" fn $new(
    // //             source: *mut IndicatorPtr<$t>,
    // //         ) -> *mut Rc<RefCell<Cross<IndicatorPtr<$t>>>> {
    // //             let source = (*source).clone();
    // //             let cross = Rc::new(RefCell::new(Cross::new(source)));
    // //             Box::into_raw(Box::new(cross))
    // //         }

    // //         #[no_mangle]
    // //         pub unsafe extern "C" fn $trait(
    // //             obj: *mut Rc<RefCell<Cross<IndicatorPtr<$t>>>>,
    // //         ) -> *mut IndicatorPtr<CrossState> {
    // //             if obj.is_null() {
    // //                 return ptr::null_mut();
    // //             }
    // //             Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
    // //         }

    // //         #[no_mangle]
    // //         pub unsafe extern "C" fn $destroy(obj: *mut Rc<RefCell<Cross<IndicatorPtr<$t>, $t>>>) {
    // //             if obj.is_null() {
    // //                 return;
    // //             }
    // //             let boxed = Box::from_raw(obj);
    // //             drop(boxed);
    // //         }
    // //     };
    // // }

    // // define_cross_methods!(
    // //     Ordering,
    // //     cross_new_f64,
    // //     cross_trait_f64,
    // //     cross_destroy_f64
    // // );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;

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
            .map(|v| Some(v))
            .collect::<Vec<_>>();
        let source_1 = VecIndicator::new(offset, source_1);
        let source_2 = VecIndicator::new(offset, source_2);
        let cross = Cross::new(source_1, source_2);

        let result = (0..10).map(|i| cross.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }
}
