use super::*;
use crate::indicator::ordering::*;

pub struct Cross<G, I> {
    source: I,
    phantom: std::marker::PhantomData<G>,
}

pub fn cross<G, V, I1, I2>(source_1: I1, source_2: I2) -> Cross<G, Ordering<G, V, I1, I2>>
where
    G: Granularity,
    V: PartialOrd,
    I1: Indicator<G, V>,
    I2: Indicator<G, V>,
{
    let source = Ordering::new(source_1, source_2);
    Cross::new(source)
}

impl<G, I> Cross<G, I> {
    pub fn new(source: I) -> Self {
        Self {
            source: source,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, I> Indicator<G, CrossState> for Cross<G, I>
where
    G: Granularity,
    I: Indicator<G, OrderingValue>,
{
    fn value(&self, time: Time<G>) -> Option<CrossState> {
        use CrossState::*;
        use OrderingValue::*;

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
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrossState {
    NotCrossed = 0,
    LtToGt = 1,
    GtToLt = -1,
}

impl Default for CrossState {
    fn default() -> Self {
        CrossState::NotCrossed
    }
}

// use std::cell::RefCell;
// use std::mem::drop;
// use std::os::raw::*;
// use std::ptr;
// use std::rc::Rc;

// // #[no_mangle]
// // pub unsafe extern "C" fn cross_new_ordering(
// //     source: *mut IndicatorPtr<OrderingValue>,
// // ) -> *mut Rc<RefCell<Cross<IndicatorPtr<OrderingValue>>>> {
// //     let source = (*source).clone();
// //     let cross = Rc::new(RefCell::new(Cross::new(source)));
// //     Box::into_raw(Box::new(cross))
// // }

// type CrossPtr<V> = Rc<RefCell<Cross<Ordering<IndicatorPtr<V>, IndicatorPtr<V>, V>>>>;

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

#[cfg(test)]
mod tests {
    use super::*;
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
            .map(|v| Some(v))
            .collect::<Vec<_>>();
        let source_1 = VecIndicator::new(offset, source_1);
        let source_2 = VecIndicator::new(offset, source_2);
        let cross = cross(source_1, source_2);

        let result = (0..10).map(|i| cross.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }
}
