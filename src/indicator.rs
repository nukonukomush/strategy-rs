use crate::time::*;
use crate::*;
use std::cell::RefCell;
use std::os::raw::*;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeValue<V> {
    Value(V),
    OutOfRange,
}

impl<V> Default for MaybeValue<V> {
    #[inline]
    fn default() -> MaybeValue<V> {
        MaybeValue::OutOfRange
    }
}

impl<V> MaybeValue<V> {
    pub fn map<T, F: FnOnce(V) -> T>(self, f: F) -> MaybeValue<T> {
        match self {
            MaybeValue::Value(x) => MaybeValue::Value(f(x)),
            MaybeValue::OutOfRange => MaybeValue::OutOfRange,
        }
    }
}

macro_rules! try_value {
    ($expr:expr) => {
        match $expr {
            MaybeValue::Value(v) => v,
            MaybeValue::OutOfRange => return MaybeValue::OutOfRange,
        }
    };
}

// Indiator は、かならず Granularity をもつ。
// Granularity の変換の際、一番小さい Granularity (1秒) にもどして相互変換する。

pub trait Indicator<G, V> {
    fn granularity(&self) -> G;
}

pub trait FuncIndicator<G, V>: Indicator<G, V> {
    fn value(&self, time: Time<G>) -> MaybeValue<V>;

    fn map<V2, F: Fn(V) -> V2>(self, f: F) -> stream::Map<G, V, V2, Self, F>
    where
        Self: Sized,
    {
        stream::Map::new(self, f)
    }

    fn zip<V2, I>(self, other: I) -> stream::Zip<G, V, V2, Self, I>
    where
        Self: Sized,
        I: FuncIndicator<G, V2>
    {
        stream::Zip::new(self, other)
    }
}

pub trait IterIndicator<G, V>: Indicator<G, V> {
    // fn into_iter(self) -> IntoIterator<Item=V>;

    fn next(&mut self) -> MaybeValue<V>;

    fn map<V2, F>(self, f: F) -> stream::Map<G, V, V2, Self, F>
    where
        Self: Sized,
        F: FnMut(V) -> V2,
    {
        stream::Map::new(self, f)
    }

    fn zip<V2, I>(self, other: I) -> stream::Zip<G, V, V2, Self, I>
    where
        Self: Sized,
        I: IterIndicator<G, V2>
    {
        stream::Zip::new(self, other)
    }
}

pub trait Provisional<G, V> {
    fn provisional_value(&self, time: Time<G>) -> MaybeValue<V>;
}

// impl<G, V> Indicator<G, V> for &dyn Indicator<G, V> {
//     #[allow(unconditional_recursion)]
//     fn value(&self, time: Time<G>) -> Option<V> {
//         self.value(time)
//     }

//     #[allow(unconditional_recursion)]
//     fn granularity(&self) -> G {
//         self.granularity()
//     }
// }

impl<G, V, I> Indicator<G, V> for RefCell<I>
where
    I: Indicator<G, V>,
{
    fn granularity(&self) -> G {
        (*self.borrow()).granularity()
    }
}

impl<G, V, I> FuncIndicator<G, V> for RefCell<I>
where
    I: FuncIndicator<G, V>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V> {
        (*self.borrow()).value(time)
    }
}

impl<G, V, I> IterIndicator<G, V> for RefCell<I>
where
    I: IterIndicator<G, V>,
{
    fn next(&mut self) -> MaybeValue<V> {
        (*self.borrow_mut()).next()
    }
}

use std::ops::Deref;
impl<G, V, I> Indicator<G, V> for Rc<I>
where
    I: Indicator<G, V>,
{
    fn granularity(&self) -> G {
        self.deref().granularity()
    }
}

impl<G, V, I> FuncIndicator<G, V> for Rc<I>
where
    I: FuncIndicator<G, V>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V> {
        self.deref().value(time)
    }
}

impl<G, V, I> IterIndicator<G, V> for Rc<RefCell<I>>
where
    I: IterIndicator<G, V>,
{
    fn next(&mut self) -> MaybeValue<V> {
        (*self.borrow_mut()).next()
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

// pub mod ffi {
//     use super::*;
//     use crate::time::ffi::*;

//     pub unsafe fn destroy<T>(ptr: *mut T) {
//         if ptr.is_null() {
//             return;
//         }
//         // ここ Box にする必要ある？？
//         let boxed = Box::from_raw(ptr);
//         drop(boxed);
//     }

//     #[derive(Clone)]
//     pub struct IndicatorPtr<V>(pub Rc<RefCell<dyn Indicator<VarGranularity, V>>>);

//     impl<V> Indicator<VarGranularity, V> for IndicatorPtr<V> {
//         fn value(&self, time: Time<VarGranularity>) -> Option<V> {
//             self.borrow().value(time)
//         }
//         fn granularity(&self) -> VarGranularity {
//             self.borrow().granularity()
//         }
//     }

//     use std::ops::Deref;
//     impl<V> Deref for IndicatorPtr<V> {
//         type Target = Rc<RefCell<dyn Indicator<VarGranularity, V>>>;
//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     macro_rules! define_value {
//         ($t:ident, $value:ident) => {
//             #[no_mangle]
//             pub unsafe extern "C" fn $value(
//                 ptr: *mut IndicatorPtr<$t>,
//                 time: CTime,
//             ) -> COption<$t> {
//                 if ptr.is_null() {
//                     return COption::none();
//                 }

//                 let ptr = &*ptr;
//                 COption::from_option(ptr.borrow().value(time.into()))
//             }
//         };
//     }
//     macro_rules! define_value_convert {
//         ($t1:ident, $t2:ident, $value:ident) => {
//             #[no_mangle]
//             pub unsafe extern "C" fn $value(
//                 ptr: *mut IndicatorPtr<$t1>,
//                 time: CTime,
//             ) -> COption<$t2> {
//                 if ptr.is_null() {
//                     return COption::none();
//                 }

//                 let ptr = &*ptr;
//                 COption::from_option(ptr.borrow().value(time.into()).map($t2::from))
//             }
//         };
//     }
//     define_value!(f64, indicator_value_f64);
//     define_value!(i32, indicator_value_i32);
//     use cross::ffi::*;
//     use cross::*;
//     define_value_convert!(CrossState, CCrossState, indicator_value_cross);
//     use crate::position::*;
//     use crate::position::ffi::*;
//     define_value_convert!(SimplePosition, CSimplePosition, indicator_value_simpleposition);
//     use trailing_stop::ffi::*;
//     use trailing_stop::*;
//     define_value_convert!(TrailingStopSignal, CTrailingStopSignal, indicator_value_trailingstopsignal);
// }

pub mod cached;
pub mod complement;
pub mod convert_granularity;
pub mod cross;
pub mod ordering;
pub mod sma;
pub mod storage;
pub mod stream;
pub mod vec;
// pub mod slope;
// pub mod trailing_stop;
