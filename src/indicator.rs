use crate::seq::*;
use crate::time::*;
use crate::*;
use approx::*;
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

    pub fn unwrap(self) -> V {
        match self {
            MaybeValue::Value(v) => v,
            MaybeValue::OutOfRange => panic!("value is out of range"),
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

impl<V> AbsDiffEq for MaybeValue<V>
where
    V: AbsDiffEq,
{
    type Epsilon = V::Epsilon;
    #[inline]
    fn default_epsilon() -> V::Epsilon {
        V::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: V::Epsilon) -> bool {
        match (self, other) {
            (MaybeValue::Value(v1), MaybeValue::Value(v2)) => V::abs_diff_eq(v1, v2, epsilon),
            (MaybeValue::OutOfRange, MaybeValue::OutOfRange) => true,
            _ => false,
        }
    }
}

impl<V> RelativeEq for MaybeValue<V>
where
    V: RelativeEq,
{
    #[inline]
    fn default_max_relative() -> V::Epsilon {
        V::default_max_relative()
    }

    #[inline]
    fn relative_eq(&self, other: &Self, epsilon: V::Epsilon, max_relative: V::Epsilon) -> bool {
        match (self, other) {
            (MaybeValue::Value(v1), MaybeValue::Value(v2)) => {
                V::relative_eq(v1, v2, epsilon, max_relative)
            }
            (MaybeValue::OutOfRange, MaybeValue::OutOfRange) => true,
            _ => false,
        }
    }
}

pub trait Indicator {
    type Seq: Sequence;
    type Val;
}

pub trait FuncIndicator: Indicator {
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val>;

    fn map<V, F>(self, f: F) -> stream::Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Val) -> V,
    {
        stream::Map::new(self, f)
    }

    fn zip<I>(self, other: I) -> stream::Zip<Self, I>
    where
        Self: Sized,
        I: FuncIndicator,
    {
        stream::Zip::new(self, other)
    }

    fn into_iter(self, offset: Self::Seq) -> stream::FuncIter<Self::Seq, Self>
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

pub trait IterIndicator: Indicator {
    fn next(&mut self) -> MaybeValue<Self::Val>;

    fn offset(&self) -> Self::Seq;

    fn map<V, F>(self, f: F) -> stream::Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Val) -> V,
    {
        stream::Map::new(self, f)
    }

    fn zip<I>(self, other: I) -> stream::Zip<Self, I>
    where
        Self: Sized,
        I: IterIndicator,
    {
        stream::Zip::new(self, other)
    }

    fn into_std(self) -> stream::StdIter<Self>
    where
        Self: Sized,
    {
        stream::StdIter::new(self)
    }

    fn into_sync_ptr(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    // fn into_storage(self) -> stream::IterStorage<Self::Seq, Self::Val, Self>
    // where
    //     Self: Sized,
    //     // Self::Seq: std::hash::Hash + Copy,
    // {
    //     stream::IterStorage::new(self)
    // }
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

// impl<S, V, I> Indicator<S, V> for RefCell<I>
// where
//     S: Sequence,
//     I: Indicator<S, V>,
// {
// }
impl<I> Indicator for RefCell<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I> FuncIndicator for RefCell<I>
where
    I: FuncIndicator,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        (*self.borrow()).value(seq)
    }
}

use std::ops::Deref;
impl<I> Indicator for Rc<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I> FuncIndicator for Rc<I>
where
    I: FuncIndicator,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.deref().value(seq)
    }
}

impl<I> Indicator for Box<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I> FuncIndicator for Box<I>
where
    I: FuncIndicator,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.deref().value(seq)
    }
}

impl<I> IterIndicator for Box<I>
where
    I: IterIndicator,
{
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.as_mut().next()
    }

    fn offset(&self) -> Self::Seq {
        self.as_ref().offset()
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

// #[cfg(ffi)]
#[macro_use]
pub mod ffi {
    use super::*;
    use crate::ffi::*;
    use crate::granularity::ffi::*;
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

    #[derive(Clone)]
    pub struct FuncIndicatorPtr<S, V>(pub Rc<RefCell<dyn FuncIndicator<Seq = S, Val = V>>>);

    impl<S, V> Indicator for FuncIndicatorPtr<S, V>
    where
        S: Sequence,
    {
        type Seq = S;
        type Val = V;
    }

    impl<S, V> FuncIndicator for FuncIndicatorPtr<S, V>
    where
        S: Sequence,
    {
        fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
            self.0.borrow().value(seq)
        }
    }

    impl<S, V> Deref for FuncIndicatorPtr<S, V> {
        type Target = Rc<RefCell<dyn FuncIndicator<Seq = S, Val = V>>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[repr(C)]
    pub struct Ptr<S, V, I> {
        pub b_ptr: *mut Rc<RefCell<I>>,
        pub f_ptr: *mut FuncIndicatorPtr<S, V>,
    }

    // pub unsafe fn indicator_value<S, CS, V>(
    //     ptr: *mut FuncIndicatorPtr<S, V>,
    //     seq: CS,
    // ) -> CMaybeValue<V>
    // where
    //     CS: Into<S>,
    //     V: Default,
    // {
    //     if ptr.is_null() {
    //         return CMaybeValue::out_of_range();
    //     }

    //     let ptr = &*ptr;
    //     CMaybeValue::from(ptr.borrow().value(seq.into()))
    // }

    pub unsafe fn value<S, CS, V, CV>(ptr: *mut FuncIndicatorPtr<S, V>, seq: CS) -> CMaybeValue<CV>
    where
        CS: Into<S>,
        CV: From<V> + Default,
    {
        if ptr.is_null() {
            return CMaybeValue::out_of_range();
        }

        let ptr = &*ptr;
        CMaybeValue::from(ptr.borrow().value(seq.into()).map(CV::from))
    }

    macro_rules! define_value {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                ptr: *mut FuncIndicatorPtr<$s, $v>,
                seq: $cs,
            ) -> CMaybeValue<$cv> {
                value(ptr, seq)
            }
        };
    }

    pub unsafe fn destroy<T>(ptr: *mut T) {
        if ptr.is_null() {
            return;
        }
        // ここ Box にする必要ある？？
        let boxed = Box::from_raw(ptr);
        drop(boxed);
    }

    macro_rules! define_destroy {
        ($ptr:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(ptr: $ptr) {
                destroy(ptr.b_ptr);
                destroy(ptr.f_ptr);
            }
        };
    }

    define_value!(GTime<Var>, CTime, f64, f64, indicator_value_time_f64);
    define_value!(GTime<Var>, CTime, i32, i32, indicator_value_time_i32);
    define_value!(
        GTime<Var>,
        CTime,
        Option<f64>,
        COption<f64>,
        indicator_value_time_option_f64
    );
    define_value!(TransactionId, i64, f64, f64, indicator_value_tid_f64);

    use cross::ffi::*;
    use cross::*;
    define_value!(
        GTime<Var>,
        CTime,
        CrossState,
        CCrossState,
        indicator_value_time_cross
    );
    define_value!(
        TransactionId,
        i64,
        CrossState,
        CCrossState,
        indicator_value_tid_cross
    );

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

pub mod balance;
pub mod cached;
pub mod complement;
pub mod convert_granularity;
pub mod convert_seq;
pub mod count;
pub mod cross;
pub mod ordering;
pub mod slope;
pub mod sma;
pub mod storage;
pub mod stream;
pub mod trade;
pub mod transaction;
pub mod vec;
// pub mod trailing_stop;
