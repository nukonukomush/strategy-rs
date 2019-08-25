use super::*;
use crate::indicator::ordering::value::Ordering as OrderingValue;
use crate::indicator::ordering::Ordering;

pub struct Cross<I> {
    source: I,
}

pub fn cross<I1, I2, V>(source_1: I1, source_2: I2) -> Cross<Ordering<I1, I2, V>>
where
    I1: Indicator<V>,
    I2: Indicator<V>,
    V: PartialOrd,
{
    let source = crate::indicator::ordering::Ordering::new(source_1, source_2);
    Cross { source: source }
}

impl<I> Cross<I> {
    pub fn new(source: I) -> Self {
        Self { source: source }
    }
}

impl<I> Indicator<CrossState> for Cross<I>
where
    I: Indicator<OrderingValue>,
{
    fn value(&self, index: isize) -> Option<CrossState> {
        use CrossState::*;
        use OrderingValue::*;

        // TODO: refactor
        if let Some(current_ord) = self.source.value(index) {
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
                let mut i = index - 1;
                while let Some(past_ord) = self.source.value(i) {
                    match (past_ord, current_ord) {
                        (Greater, Less) => return Some(GtToLt),
                        (Less, Greater) => return Some(LtToGt),
                        (Greater, Greater) => return Some(NotCrossed),
                        (Less, Less) => return Some(NotCrossed),
                        _ => (),
                    }
                    i -= 1;
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

use std::cell::RefCell;
use std::mem::drop;
use std::os::raw::*;
use std::ptr;
use std::rc::Rc;

// #[no_mangle]
// pub unsafe extern "C" fn cross_new_ordering(
//     source: *mut IndicatorPtr<OrderingValue>,
// ) -> *mut Rc<RefCell<Cross<IndicatorPtr<OrderingValue>>>> {
//     let source = (*source).clone();
//     let cross = Rc::new(RefCell::new(Cross::new(source)));
//     Box::into_raw(Box::new(cross))
// }

type CrossPtr<V> = Rc<RefCell<Cross<Ordering<IndicatorPtr<V>, IndicatorPtr<V>, V>>>>;

#[no_mangle]
pub unsafe extern "C" fn cross_new_f64(
    source_1: *mut IndicatorPtr<f64>,
    source_2: *mut IndicatorPtr<f64>,
) -> *mut CrossPtr<f64> {
    let source_1 = (*source_1).clone();
    let source_2 = (*source_2).clone();
    let cross = Rc::new(RefCell::new(cross(source_1, source_2)));
    Box::into_raw(Box::new(cross))
}

#[no_mangle]
pub unsafe extern "C" fn cross_trait_f64(obj: *mut CrossPtr<f64>) -> *mut IndicatorPtr<CrossState> {
    if obj.is_null() {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
}

#[no_mangle]
pub unsafe extern "C" fn cross_destroy_f64(obj: *mut CrossPtr<f64>) {
    if obj.is_null() {
        return;
    }
    let boxed = Box::from_raw(obj);
    drop(boxed);
}

// #[no_mangle]
// pub unsafe extern "C" fn cross_trait_ordering(
//     obj: *mut Rc<RefCell<Cross<IndicatorPtr<OrderingValue>>>>,
// ) -> *mut IndicatorPtr<CrossState> {
//     if obj.is_null() {
//         return ptr::null_mut();
//     }
//     Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
// }

// #[no_mangle]
// pub unsafe extern "C" fn cross_destroy_ordering(
//     obj: *mut Rc<RefCell<Cross<IndicatorPtr<OrderingValue>>>>,
// ) {
//     if obj.is_null() {
//         return;
//     }
//     let boxed = Box::from_raw(obj);
//     drop(boxed);
// }

// macro_rules! define_cross_methods {
//     ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
//         #[no_mangle]
//         pub unsafe extern "C" fn $new(
//             source: *mut IndicatorPtr<$t>,
//         ) -> *mut Rc<RefCell<Cross<IndicatorPtr<$t>>>> {
//             let source = (*source).clone();
//             let cross = Rc::new(RefCell::new(Cross::new(source)));
//             Box::into_raw(Box::new(cross))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $trait(
//             obj: *mut Rc<RefCell<Cross<IndicatorPtr<$t>>>>,
//         ) -> *mut IndicatorPtr<CrossState> {
//             if obj.is_null() {
//                 return ptr::null_mut();
//             }
//             Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $destroy(obj: *mut Rc<RefCell<Cross<IndicatorPtr<$t>, $t>>>) {
//             if obj.is_null() {
//                 return;
//             }
//             let boxed = Box::from_raw(obj);
//             drop(boxed);
//         }
//     };
// }

// define_cross_methods!(
//     Ordering,
//     cross_new_f64,
//     cross_trait_f64,
//     cross_destroy_f64
// );

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicator::ordering::*;
    use crate::indicator::tests::*;

    #[test]
    fn test_cross() {
        use CrossState::GtToLt as gtl;
        use CrossState::LtToGt as ltg;
        use CrossState::NotCrossed as not;

        let source_1 = vec![0.0, 0.0, 2.0, 2.0, 0.0, 1.0, 1.0, 2.0, 1.0, 0.0];
        let source_2 = vec![1.0; 10];
        let expected = vec![not, not, ltg, not, gtl, not, not, ltg, not, gtl];
        // let ordering = crate::indicator::ordering::Ordering::new(source_1, source_2);
        // let cross = Cross::new(ordering);
        let cross = cross(source_1, source_2);

        let result = indicator_iter(cross).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }
}
