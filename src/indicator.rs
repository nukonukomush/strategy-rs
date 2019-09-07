use crate::seq::*;
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

pub trait Indicator<S, V> {}

pub trait FuncIndicator<S, V>: Indicator<S, V> {
    fn value(&self, seq: S) -> MaybeValue<V>;

    fn map<V2, F>(self, f: F) -> stream::Map<S, V, V2, Self, F>
    where
        Self: Sized,
        F: Fn(V) -> V2,
    {
        stream::Map::new(self, f)
    }

    fn zip<V2, I>(self, other: I) -> stream::Zip<S, V, V2, Self, I>
    where
        Self: Sized,
        I: FuncIndicator<S, V2>,
    {
        stream::Zip::new(self, other)
    }

    fn into_iter(self, offset: S) -> stream::FuncIter<S, Self>
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

pub trait IterIndicator<S, V>: Indicator<S, V> {
    fn next(&mut self) -> MaybeValue<V>;

    fn offset(&self) -> S;

    fn map<V2, F>(self, f: F) -> stream::Map<S, V, V2, Self, F>
    where
        Self: Sized,
        F: FnMut(V) -> V2,
    {
        stream::Map::new(self, f)
    }

    fn zip<V2, I>(self, other: I) -> stream::Zip<S, V, V2, Self, I>
    where
        Self: Sized,
        I: IterIndicator<S, V2>,
    {
        stream::Zip::new(self, other)
    }

    fn into_std(self) -> stream::StdIter<S, V, Self>
    where
        Self: Sized,
    {
        stream::StdIter::new(self)
    }

    fn into_storage(self) -> stream::IterStorage<S, V, Self>
    where
        Self: Sized,
        S: Sequence + std::hash::Hash + Copy,
    {
        stream::IterStorage::new(self)
    }
}

pub trait Provisional<S, V>
where
    S: Sequence,
{
    fn provisional_value(&self, seq: S) -> MaybeValue<V>;
}

// impl<S, V> Indicator<S, V> for &dyn Indicator<S, V> {
//     #[allow(unconditional_recursion)]
//     fn value(&self, time: Sequence) -> Option<V> {
//         self.value(time)
//     }

//     #[allow(unconditional_recursion)]
//     fn granularity(&self) -> G {
//         self.granularity()
//     }
// }
//

impl<S, V, I> Indicator<S, V> for RefCell<I>
where
    S: Sequence,
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> G {
    //     (*self.borrow()).granularity()
    // }
}

impl<S, V, I> FuncIndicator<S, V> for RefCell<I>
where
    S: Sequence,
    I: FuncIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        (*self.borrow()).value(seq)
    }
}

use std::ops::Deref;
impl<S, V, I> Indicator<S, V> for Rc<I>
where
    S: Sequence,
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> G {
    //     self.deref().granularity()
    // }
}

impl<S, V, I> FuncIndicator<S, V> for Rc<I>
where
    S: Sequence,
    I: FuncIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        self.deref().value(seq)
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

    // pub fn indicator_iter<S, V, I>(indicator: I) -> impl Iterator<Item = V>
    // where
    //     I: Indicator<S, V>,
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

#[cfg(ffi)]
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
    pub struct FuncIndicatorPtr<S, V>(pub Rc<RefCell<dyn FuncIndicator<S, V>>>);

    type G = VarGranularity;

    impl<S, V> Indicator<S, V> for FuncIndicatorPtr<S, V> {
        // fn granularity(&self) -> G {
        //     self.0.borrow().granularity()
        // }
    }

    impl<S, V> FuncIndicator<S, V> for FuncIndicatorPtr<S, V> {
        fn value(&self, seq: S) -> MaybeValue<V> {
            self.0.borrow().value(seq)
        }
    }

    impl<S, V> Deref for FuncIndicatorPtr<S, V> {
        type Target = Rc<RefCell<dyn FuncIndicator<S, V>>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[repr(C)]
    pub struct Ptr<S, V, I> {
        pub b_ptr: *mut Rc<RefCell<I>>,
        pub f_ptr: *mut FuncIndicatorPtr<S, V>,
    }

    macro_rules! define_value {
        ($t:ty, $value:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $value(
                ptr: *mut FuncIndicatorPtr<Time<G>, $t>,
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
                ptr: *mut FuncIndicatorPtr<Time<G>, $t1>,
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

#[cfg(ffi)]
pub mod ffi_iter {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;
    use stream::*;

    type G = VarGranularity;
    type IPtr<Sq, V> = Ptr<Sq, V, IterVec<S, V, FuncIter<S, FuncIndicatorPtr<V>>>>;
    // type IPtr<V> = Ptr<V, IterVec<S, V, Map<S, V, V, kjFuncIter<S, FuncIndicatorPtr<V>>>>;

    // pub struct CallBack<V1, V2> {
    //     cb: extern "C" fn(V1) -> V2,
    // }

    // impl<V1, V2> FnMut(Args) for CallBack<V1, V2> {
    // }

    macro_rules! define_via_iter_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                source: *mut FuncIndicatorPtr<$t>,
                offset: CTime,
            ) -> IPtr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(IterVec::new(FuncIter::new(
                    source,
                    offset.into(),
                ))));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: IPtr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.f_ptr);
            }
        };
    }

    define_via_iter_methods!(f64, via_iter_new_f64, via_iter_destroy_f64);
}

pub mod cached;
pub mod complement;
pub mod convert_granularity;
pub mod cross;
pub mod ordering;
pub mod slope;
pub mod sma;
pub mod storage;
pub mod stream;
pub mod vec;
// pub mod trailing_stop;
pub mod count;
// pub mod transaction;
