use crate::*;
use std::cell::RefCell;
use std::os::raw::*;
use std::rc::Rc;

pub trait Indicator<T> {
    fn value(&self, index: isize) -> Option<T>;
}

#[derive(Clone)]
pub struct IndicatorPtr<T>(pub Rc<RefCell<dyn Indicator<T>>>);

impl<T> Indicator<T> for IndicatorPtr<T>
{
    fn value(&self, index: isize) -> Option<T> {
        self.borrow().value(index)
    }
}

use std::ops::Deref;
impl<T> Deref for IndicatorPtr<T>
{
    type Target = Rc<RefCell<dyn Indicator<T>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, U> Indicator<T> for RefCell<U>
where
    T: Clone,
    U: Indicator<T>,
{
    fn value(&self, index: isize) -> Option<T> {
        let inner = self.borrow();
        (*inner).value(index)
    }
}

impl<T, U> Indicator<T> for Rc<U>
where
    T: Clone,
    U: Indicator<T>,
{
    #[allow(unconditional_recursion)]
    fn value(&self, index: isize) -> Option<T> {
        self.value(index)
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

    pub fn indicator_iter<T, U>(indicator: T) -> impl Iterator<Item = U>
    where
        T: Indicator<U>,
    {
        let mut index = 0;
        let f = move || {
            let value = indicator.value(index);
            index += 1;
            value
        };
        FnMutIter {
            closure: f,
            phantom: std::marker::PhantomData,
        }
    }
}

pub mod cached;
pub mod vec;
pub mod sma;
pub mod ordering;
pub mod cross;

#[no_mangle]
pub unsafe extern "C" fn indicator_value_f64(
    ptr: *mut IndicatorPtr<f64>,
    i: c_int,
) -> COption<f64> {
    if ptr.is_null() {
        return COption::none();
    }

    let ptr = &*ptr;
    COption::from_option(ptr.borrow().value(i as isize))
}

#[no_mangle]
pub unsafe extern "C" fn indicator_destroy_f64(obj: *mut IndicatorPtr<f64>) {
    if obj.is_null() {
        return;
    }
    let boxed = Box::from_raw(obj);
    drop(boxed);
}

use cross::CrossState;
#[no_mangle]
pub unsafe extern "C" fn indicator_value_cross(
    ptr: *mut IndicatorPtr<CrossState>,
    i: c_int,
) -> COption<CrossState> {
    if ptr.is_null() {
        return COption::none();
    }

    let ptr = &*ptr;
    COption::from_option(ptr.borrow().value(i as isize))
}

#[no_mangle]
pub unsafe extern "C" fn indicator_destroy_cross(obj: *mut IndicatorPtr<CrossState>) {
    if obj.is_null() {
        return;
    }
    let boxed = Box::from_raw(obj);
    drop(boxed);
}
