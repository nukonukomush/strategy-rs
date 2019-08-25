use crate::*;
use crate::time::*;
use std::cell::RefCell;
use std::os::raw::*;
use std::rc::Rc;

pub trait Indicator<G, V> {
    fn value(&self, time: Time<G>) -> Option<V>;
}

#[derive(Clone)]
pub struct IndicatorPtr<G, V>(pub Rc<RefCell<dyn Indicator<G, V>>>);

impl<G, V> Indicator<G, V> for IndicatorPtr<G, V> {
    fn value(&self, time: Time<G>) -> Option<V> {
        self.borrow().value(time)
    }
}

use std::ops::Deref;
impl<G, V> Deref for IndicatorPtr<G, V> {
    type Target = Rc<RefCell<dyn Indicator<G, V>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G, V, I> Indicator<G, V> for RefCell<I>
where
    V: Clone,
    I: Indicator<G, V>,
{
    fn value(&self, time: Time<G>) -> Option<V> {
        let inner = self.borrow();
        (*inner).value(time)
    }
}

impl<G, V, I> Indicator<G, V> for Rc<I>
where
    V: Clone,
    I: Indicator<G, V>,
{
    #[allow(unconditional_recursion)]
    fn value(&self, time: Time<G>) -> Option<V> {
        self.value(time)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct FnMutIter<F, T> {
        closure: F,
        phantom: std::marker::PhantomData<T>,
    }

    impl<F, T> std::iter::Iterator for FnMutIter<F, T>
    where
        F: FnMut() -> Option<T>,
    {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            (self.closure)()
        }
    }

    // pub fn indicator_iter<G, V, I>(indicator: I) -> impl Iterator<Item = V>
    // where
    //     I: Indicator<G, V>,
    // {
    //     let mut index = 0;
    //     let f = move || {
    //         let value = indicator.value(index);
    //         index += 1;
    //         value
    //     };
    //     FnMutIter {
    //         closure: f,
    //         phantom: std::marker::PhantomData,
    //     }
    // }
}

pub mod cached;
pub mod vec;
pub mod convert_granularity;
pub mod sma;
// pub mod ordering;
// pub mod cross;

// #[no_mangle]
// pub unsafe extern "C" fn indicator_value_f64(
//     ptr: *mut IndicatorPtr<f64>,
//     i: c_int,
// ) -> COption<f64> {
//     if ptr.is_null() {
//         return COption::none();
//     }

//     let ptr = &*ptr;
//     COption::from_option(ptr.borrow().value(i as isize))
// }

// #[no_mangle]
// pub unsafe extern "C" fn indicator_destroy_f64(obj: *mut IndicatorPtr<f64>) {
//     if obj.is_null() {
//         return;
//     }
//     let boxed = Box::from_raw(obj);
//     drop(boxed);
// }

// use cross::CrossState;
// #[no_mangle]
// pub unsafe extern "C" fn indicator_value_cross(
//     ptr: *mut IndicatorPtr<CrossState>,
//     i: c_int,
// ) -> COption<CrossState> {
//     if ptr.is_null() {
//         return COption::none();
//     }

//     let ptr = &*ptr;
//     COption::from_option(ptr.borrow().value(i as isize))
// }

// #[no_mangle]
// pub unsafe extern "C" fn indicator_destroy_cross(obj: *mut IndicatorPtr<CrossState>) {
//     if obj.is_null() {
//         return;
//     }
//     let boxed = Box::from_raw(obj);
//     drop(boxed);
// }
