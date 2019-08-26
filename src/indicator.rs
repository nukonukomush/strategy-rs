use crate::time::*;
use crate::*;
use std::cell::RefCell;
use std::os::raw::*;
use std::rc::Rc;

pub trait Indicator<G, V> {
    fn value(&self, time: Time<G>) -> Option<V>;
    fn granularity(&self) -> G;
}

impl<G, V> Indicator<G, V> for &dyn Indicator<G, V> {
    #[allow(unconditional_recursion)]
    fn value(&self, time: Time<G>) -> Option<V> {
        self.value(time)
    }

    #[allow(unconditional_recursion)]
    fn granularity(&self) -> G {
        self.granularity()
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
    fn granularity(&self) -> G {
        let inner = self.borrow();
        (*inner).granularity()
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
    #[allow(unconditional_recursion)]
    fn granularity(&self) -> G {
        self.granularity()
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

pub mod ffi {
    use super::*;
    use crate::time::ffi::*;

    pub unsafe fn destroy<T>(ptr: *mut T) {
        if ptr.is_null() {
            return;
        }
        // ここ Box にする必要ある？？
        let boxed = Box::from_raw(ptr);
        drop(boxed);
    }

    #[derive(Clone)]
    pub struct IndicatorPtr<V>(pub Rc<RefCell<dyn Indicator<VarGranularity, V>>>);

    impl<V> Indicator<VarGranularity, V> for IndicatorPtr<V> {
        fn value(&self, time: Time<VarGranularity>) -> Option<V> {
            self.borrow().value(time)
        }
        fn granularity(&self) -> VarGranularity {
            self.borrow().granularity()
        }
    }

    use std::ops::Deref;
    impl<V> Deref for IndicatorPtr<V> {
        type Target = Rc<RefCell<dyn Indicator<VarGranularity, V>>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    macro_rules! define_value {
        ($t:ident, $value:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $value(
                ptr: *mut IndicatorPtr<$t>,
                time: CTime,
            ) -> COption<$t> {
                if ptr.is_null() {
                    return COption::none();
                }

                let ptr = &*ptr;
                COption::from_option(ptr.borrow().value(time.into()))
            }
        };
    }
    macro_rules! define_value_convert {
        ($t1:ident, $t2:ident, $value:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $value(
                ptr: *mut IndicatorPtr<$t1>,
                time: CTime,
            ) -> COption<$t2> {
                if ptr.is_null() {
                    return COption::none();
                }

                let ptr = &*ptr;
                COption::from_option(ptr.borrow().value(time.into()).map($t2::from))
            }
        };
    }
    define_value!(f64, indicator_value_f64);
    use cross::ffi::*;
    use cross::*;
    define_value_convert!(CrossState, CCrossState, indicator_value_cross);
}

pub mod cached;
pub mod stream;
pub mod complement;
pub mod convert_granularity;
pub mod cross;
pub mod ordering;
pub mod sma;
pub mod vec;
pub mod func;
