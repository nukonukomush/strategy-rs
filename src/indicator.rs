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

impl<V> Into<Option<V>> for MaybeValue<V> {
    fn into(self) -> Option<V> {
        match self {
            MaybeValue::Value(x) => Some(x),
            MaybeValue::OutOfRange => None,
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

    fn map<V2, F>(self, f: F) -> stream::Map<G, V, V2, Self, F>
    where
        Self: Sized,
        F: Fn(V) -> V2,
    {
        stream::Map::new(self, f)
    }

    fn zip<V2, I>(self, other: I) -> stream::Zip<G, V, V2, Self, I>
    where
        Self: Sized,
        I: FuncIndicator<G, V2>,
    {
        stream::Zip::new(self, other)
    }

    fn into_iter(self, offset: Time<G>) -> stream::FuncIter<G, Self>
    where
        Self: Sized,
    {
        stream::FuncIter::new(self, offset)
    }

    fn into_sync_ptr(self) -> Rc<RefCell<Self>>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(self))
    }
}

pub trait IterIndicator<G, V>: Indicator<G, V> {
    fn next(&mut self) -> MaybeValue<V>;

    fn offset(&self) -> Time<G>;

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
        I: IterIndicator<G, V2>,
    {
        stream::Zip::new(self, other)
    }

    fn into_std(self) -> stream::StdIter<G, V, Self>
    where
        Self: Sized,
    {
        stream::StdIter::new(self)
    }

    fn into_storage(self) -> stream::IterStorage<G, V, Self>
    where
        Self: Sized,
        G: Granularity + Eq + std::hash::Hash + Copy + Ord,
    {
        stream::IterStorage::new(self)
    }

    fn into_sync_ptr(self) -> Rc<RefCell<Self>>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(self))
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

    fn offset(&self) -> Time<G> {
        (*self.borrow_mut()).offset()
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

    fn offset(&self) -> Time<G> {
        (*self.borrow_mut()).offset()
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

// #[cfg(ffi)]
pub mod ffi {
    use super::*;
    use crate::ffi::*;
    use crate::time::ffi::*;
    use std::ops::Deref;

    #[repr(C)]
    pub struct CMaybeValue<T> {
        is_value: c_char,
        value: T,
    }

    impl<T> CMaybeValue<T>
    where
        T: Default,
    {
        pub fn out_of_range() -> Self {
            Self {
                is_value: 0,
                value: Default::default(),
            }
        }

        pub fn value(value: T) -> Self {
            Self {
                is_value: 1,
                value: value,
            }
        }

        pub fn from_option(value: Option<T>) -> Self {
            match value {
                Some(value) => Self::value(value),
                None => Self::out_of_range(),
            }
        }
    }

    impl<V> From<MaybeValue<V>> for CMaybeValue<V>
    where
        V: Default,
    {
        fn from(v: MaybeValue<V>) -> Self {
            match v {
                MaybeValue::Value(v) => CMaybeValue::value(v),
                MaybeValue::OutOfRange => CMaybeValue::out_of_range(),
            }
        }
    }

    pub unsafe fn destroy<T>(ptr: *mut T) {
        if ptr.is_null() {
            return;
        }
        // ここ Box にする必要ある？？
        let boxed = Box::from_raw(ptr);
        drop(boxed);
    }

    #[derive(Clone)]
    pub struct FuncIndicatorPtr<V>(pub Rc<RefCell<dyn FuncIndicator<VarGranularity, V>>>);

    impl<V> Indicator<VarGranularity, V> for FuncIndicatorPtr<V> {
        fn granularity(&self) -> VarGranularity {
            self.granularity()
        }
    }

    impl<V> FuncIndicator<VarGranularity, V> for FuncIndicatorPtr<V> {
        fn value(&self, time: Time<VarGranularity>) -> MaybeValue<V> {
            self.value(time)
        }
    }

    impl<V> Deref for FuncIndicatorPtr<V> {
        type Target = Rc<RefCell<dyn FuncIndicator<VarGranularity, V>>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Clone)]
    pub struct IterIndicatorPtr<V>(pub Rc<RefCell<dyn IterIndicator<VarGranularity, V>>>);

    impl<V> Indicator<VarGranularity, V> for IterIndicatorPtr<V> {
        fn granularity(&self) -> VarGranularity {
            self.granularity()
        }
    }

    impl<V> IterIndicator<VarGranularity, V> for IterIndicatorPtr<V> {
        fn next(&mut self) -> MaybeValue<V> {
            self.next()
        }

        fn offset(&self) -> Time<VarGranularity> {
            self.offset()
        }
    }

    impl<V> Deref for IterIndicatorPtr<V> {
        type Target = Rc<RefCell<dyn IterIndicator<VarGranularity, V>>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[repr(C)]
    pub struct Ptr<V, I> {
        pub b_ptr: *mut Rc<RefCell<I>>,
        pub f_ptr: *mut FuncIndicatorPtr<V>,
        pub i_ptr: *mut IterIndicatorPtr<V>,
    }

    macro_rules! define_value {
        ($t:ty, $value:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $value(
                ptr: *mut FuncIndicatorPtr<$t>,
                time: CTime,
            ) -> CMaybeValue<$t> {
                if ptr.is_null() {
                    return CMaybeValue::out_of_range();
                }

                let ptr = &*ptr;
                CMaybeValue::from(ptr.borrow().value(time.into()))
            }
        };
    }
    macro_rules! define_value_convert {
        ($t1:ty, $t2:ty, $value:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $value(
                ptr: *mut FuncIndicatorPtr<$t1>,
                time: CTime,
            ) -> CMaybeValue<$t2> {
                if ptr.is_null() {
                    return CMaybeValue::out_of_range();
                }

                let ptr = &*ptr;
                CMaybeValue::from(ptr.borrow().value(time.into()).map(<$t2>::from))
            }
        };
    }
    define_value!(f64, indicator_value_f64);
    define_value!(i32, indicator_value_i32);
    define_value_convert!(Option<f64>, COption<f64>, indicator_value_option_f64);
    // use cross::ffi::*;
    // use cross::*;
    // define_value_convert!(CrossState, CCrossState, indicator_value_cross);
    // use crate::position::ffi::*;
    // use crate::position::*;
    // define_value_convert!(
    //     SimplePosition,
    //     CSimplePosition,
    //     indicator_value_simpleposition
    // );
    // use trailing_stop::ffi::*;
    // use trailing_stop::*;
    // define_value_convert!(
    //     TrailingStopSignal,
    //     CTrailingStopSignal,
    //     indicator_value_trailingstopsignal
    // );
}

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
