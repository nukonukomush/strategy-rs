use super::*;
use crate::time::*;
use crate::*;

pub struct Sma<G, I> {
    source: I,
    period: isize,
    phantom: std::marker::PhantomData<G>,
}

impl<G, I> Sma<G, I> {
    pub fn new(source: I, period: usize) -> Self {
        Self {
            source: source,
            period: period as isize,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, I> Indicator<G, f64> for Sma<G, I>
where
    G: Granularity,
    I: Indicator<G, f64>,
{
    fn value(&self, time: Time<G>) -> Option<f64> {
        let mut sum = 0.0;
        let begin = time + 1 - (self.period as i64);
        // for i in (begin..=time).rev() {
        //     let v = self.source.value(i)?;
        //     sum += v;
        // }
        let mut tmp = time;
        while tmp >= begin {
            let v = self.source.value(tmp)?;
            sum += v;
            tmp = tmp - 1;
        }
        Some(sum / self.period as f64)
    }
}

// use std::cell::RefCell;
// use std::mem::drop;
// use std::os::raw::*;
// use std::ptr;
// use std::rc::Rc;

// macro_rules! define_sma_methods {
//     ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
//         #[no_mangle]
//         pub unsafe extern "C" fn $new(
//             source: *mut IndicatorPtr<$t>,
//             period: c_int,
//         ) -> *mut Rc<RefCell<Sma<IndicatorPtr<$t>>>> {
//             let source = (*source).clone();
//             let sma = Rc::new(RefCell::new(Sma::new(source, period as usize)));
//             Box::into_raw(Box::new(sma))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $trait(
//             obj: *mut Rc<RefCell<Sma<IndicatorPtr<$t>>>>,
//         ) -> *mut IndicatorPtr<$t> {
//             if obj.is_null() {
//                 return ptr::null_mut();
//             }
//             Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $destroy(
//             obj: *mut Rc<RefCell<Sma<IndicatorPtr<$t>>>>,
//         ) {
//             if obj.is_null() {
//                 return;
//             }
//             let boxed = Box::from_raw(obj);
//             drop(boxed);
//         }
//     };
// }

// define_sma_methods!(f64, sma_new_f64, sma_trait_f64, sma_destroy_f64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    // use crate::indicator::cached::*;
    // use crate::indicator::tests::*;

    #[test]
    fn test_sma() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![None, None, Some(2.0), Some(3.0), Some(4.0)];
        // let sma_pre = Sma::new(source, 3);
        // let sma = Cached::new(sma_pre);
        let hash = vec(offset, source.clone());
        let sma = Sma::new(hash, 3);

        let result = (0..5).map(|i| sma.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
