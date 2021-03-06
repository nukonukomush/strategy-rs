use crate::seq::*;
use crate::time::*;
use crate::*;
use approx::*;
use std::cell::RefCell;
use std::ops::Deref;
use std::os::raw::*;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeFixed<T> {
    Fixed(T),
    NotFixed,
}

macro_rules! try_fixed {
    ($expr:expr) => {
        match $expr {
            MaybeFixed::Fixed(v) => v,
            MaybeFixed::NotFixed => return MaybeFixed::NotFixed,
        }
    };
}

impl<V> MaybeFixed<V> {
    #[inline]
    pub fn map<T, F: FnOnce(V) -> T>(self, f: F) -> MaybeFixed<T> {
        match self {
            MaybeFixed::Fixed(x) => MaybeFixed::Fixed(f(x)),
            MaybeFixed::NotFixed => MaybeFixed::NotFixed,
        }
    }

    #[inline]
    pub fn unwrap(self) -> V {
        match self {
            MaybeFixed::Fixed(v) => v,
            MaybeFixed::NotFixed => panic!("value is not fixed"),
        }
    }

    #[inline]
    pub fn is_fixed(&self) -> bool {
        match self {
            MaybeFixed::Fixed(_) => true,
            MaybeFixed::NotFixed => false,
        }
    }

    #[inline]
    pub fn is_not_fixed(&self) -> bool {
        match self {
            MaybeFixed::Fixed(_) => false,
            MaybeFixed::NotFixed => true,
        }
    }

    #[inline]
    pub fn zip<V2>(self, other: MaybeFixed<V2>) -> MaybeFixed<(V, V2)> {
        let v1 = try_fixed!(self);
        let v2 = try_fixed!(other);
        MaybeFixed::Fixed((v1, v2))
    }
}

impl<V> MaybeFixed<MaybeInRange<V>> {
    #[inline]
    pub fn map2<T, F: FnOnce(V) -> T>(self, f: F) -> MaybeFixed<MaybeInRange<T>> {
        match self {
            MaybeFixed::Fixed(MaybeInRange::InRange(x)) => {
                MaybeFixed::Fixed(MaybeInRange::InRange(f(x)))
            }
            MaybeFixed::Fixed(MaybeInRange::OutOfRange) => {
                MaybeFixed::Fixed(MaybeInRange::OutOfRange)
            }
            MaybeFixed::NotFixed => MaybeFixed::NotFixed,
        }
    }

    #[inline]
    pub fn zip2<T>(self, other: MaybeFixed<MaybeInRange<T>>) -> MaybeFixed<MaybeInRange<(V, T)>> {
        self.and_then(|v1| other.map2(|v2| (v1, v2)))
    }

    #[inline]
    pub fn and_then<T, F: FnOnce(V) -> MaybeFixed<MaybeInRange<T>>>(
        self,
        f: F,
    ) -> MaybeFixed<MaybeInRange<T>> {
        match self {
            MaybeFixed::Fixed(MaybeInRange::InRange(x)) => f(x),
            MaybeFixed::Fixed(MaybeInRange::OutOfRange) => {
                MaybeFixed::Fixed(MaybeInRange::OutOfRange)
            }
            MaybeFixed::NotFixed => MaybeFixed::NotFixed,
        }
    }

    #[inline]
    pub fn when_not_fixed<F: FnOnce() -> MaybeFixed<MaybeInRange<V>>>(
        self,
        f: F,
    ) -> MaybeFixed<MaybeInRange<V>> {
        match self {
            MaybeFixed::NotFixed => f(),
            other => other,
        }
    }

    #[inline]
    pub fn when_out_of_range<F: FnOnce() -> MaybeFixed<MaybeInRange<V>>>(
        self,
        f: F,
    ) -> MaybeFixed<MaybeInRange<V>> {
        match self {
            MaybeFixed::Fixed(MaybeInRange::OutOfRange) => f(),
            other => other,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeInRange<T> {
    InRange(T),
    OutOfRange,
}

macro_rules! try_in_range {
    ($expr:expr) => {
        match $expr {
            MaybeInRange::InRange(v) => v,
            MaybeInRange::OutOfRange => return MaybeInRange::OutOfRange,
        }
    };
}

impl<V> MaybeInRange<V> {
    #[inline]
    pub fn map<T, F: FnOnce(V) -> T>(self, f: F) -> MaybeInRange<T> {
        match self {
            MaybeInRange::InRange(x) => MaybeInRange::InRange(f(x)),
            MaybeInRange::OutOfRange => MaybeInRange::OutOfRange,
        }
    }

    #[inline]
    pub fn unwrap(self) -> V {
        match self {
            MaybeInRange::InRange(v) => v,
            MaybeInRange::OutOfRange => panic!("value is out of range"),
        }
    }

    #[inline]
    pub fn is_in_range(&self) -> bool {
        match self {
            MaybeInRange::InRange(_) => true,
            MaybeInRange::OutOfRange => false,
        }
    }

    #[inline]
    pub fn is_out_of_range(&self) -> bool {
        match self {
            MaybeInRange::InRange(_) => false,
            MaybeInRange::OutOfRange => true,
        }
    }

    #[inline]
    pub fn zip<V2>(self, other: MaybeInRange<V2>) -> MaybeInRange<(V, V2)> {
        let v1 = try_in_range!(self);
        let v2 = try_in_range!(other);
        MaybeInRange::InRange((v1, v2))
    }
}

#[macro_export]
macro_rules! try_value {
    ($expr:expr) => {
        match $expr {
            MaybeFixed::Fixed(MaybeInRange::InRange(v)) => v,
            MaybeFixed::Fixed(MaybeInRange::OutOfRange) => {
                return MaybeFixed::Fixed(MaybeInRange::OutOfRange)
            }
            MaybeFixed::NotFixed => return MaybeFixed::NotFixed,
        }
    };
}

pub type MaybeValue<T> = MaybeFixed<MaybeInRange<T>>;

impl<V> AbsDiffEq for MaybeFixed<V>
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
            (MaybeFixed::Fixed(v1), MaybeFixed::Fixed(v2)) => V::abs_diff_eq(v1, v2, epsilon),
            (MaybeFixed::NotFixed, MaybeFixed::NotFixed) => true,
            _ => false,
        }
    }
}

impl<V> RelativeEq for MaybeFixed<V>
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
            (MaybeFixed::Fixed(v1), MaybeFixed::Fixed(v2)) => {
                V::relative_eq(v1, v2, epsilon, max_relative)
            }
            (MaybeFixed::NotFixed, MaybeFixed::NotFixed) => true,
            _ => false,
        }
    }
}

impl<V> AbsDiffEq for MaybeInRange<V>
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
            (MaybeInRange::InRange(v1), MaybeInRange::InRange(v2)) => {
                V::abs_diff_eq(v1, v2, epsilon)
            }
            (MaybeInRange::OutOfRange, MaybeInRange::OutOfRange) => true,
            _ => false,
        }
    }
}

impl<V> RelativeEq for MaybeInRange<V>
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
            (MaybeInRange::InRange(v1), MaybeInRange::InRange(v2)) => {
                V::relative_eq(v1, v2, epsilon, max_relative)
            }
            (MaybeInRange::OutOfRange, MaybeInRange::OutOfRange) => true,
            _ => false,
        }
    }
}

pub trait Indicator {
    type Seq: Sequence;
    type Val: std::fmt::Debug;
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

    fn then<V, F>(self, f: F) -> stream::Then<Self, F>
    where
        Self: Sized,
        F: Fn(MaybeValue<Self::Val>) -> MaybeValue<V>,
    {
        stream::Then::new(self, f)
    }

    fn and_then<V, F>(self, f: F) -> stream::AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Val) -> MaybeValue<V>,
    {
        stream::AndThen::new(self, f)
    }

    fn when_not_fixed<V, F>(self, f: F) -> stream::WhenNotFixed<Self, F>
    where
        Self: Sized,
        F: Fn() -> MaybeValue<V>,
    {
        stream::WhenNotFixed::new(self, f)
    }

    fn when_out_of_range<V, F>(self, f: F) -> stream::WhenOutOfRange<Self, F>
    where
        Self: Sized,
        F: Fn() -> MaybeValue<V>,
    {
        stream::WhenOutOfRange::new(self, f)
    }

    fn zip<I>(self, other: I) -> stream::FuncZip<Self, I>
    where
        Self: Sized,
        I: FuncIndicator,
    {
        stream::FuncZip::new(self, other)
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

    fn rolling<V, F>(self, size: usize, f: F) -> rolling::Rolling<Self, F>
    where
        Self: Sized,
        F: Fn(rolling::FixedSizeWindow<Self::Seq, Self>) -> MaybeValue<V>,
    {
        rolling::Rolling::new(self, size, f)
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

    fn then<V, F>(self, f: F) -> stream::Then<Self, F>
    where
        Self: Sized,
        F: FnMut(MaybeValue<Self::Val>) -> MaybeValue<V>,
    {
        stream::Then::new(self, f)
    }

    fn and_then<V, F>(self, f: F) -> stream::AndThen<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Val) -> MaybeValue<V>,
    {
        stream::AndThen::new(self, f)
    }

    fn when_not_fixed<V, F>(self, f: F) -> stream::WhenNotFixed<Self, F>
    where
        Self: Sized,
        F: FnMut() -> MaybeValue<V>,
    {
        stream::WhenNotFixed::new(self, f)
    }

    fn when_out_of_range<V, F>(self, f: F) -> stream::WhenOutOfRange<Self, F>
    where
        Self: Sized,
        F: FnMut() -> MaybeValue<V>,
    {
        stream::WhenOutOfRange::new(self, f)
    }

    fn zip<I>(self, other: I) -> stream::IterZip<Self::Val, I::Val, Self, I>
    where
        Self: Sized,
        I: IterIndicator,
    {
        stream::IterZip::new(self, other)
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
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        (*self.borrow()).value(seq)
    }
}

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
    #[inline]
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
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.deref().value(seq)
    }
}

impl<I> IterIndicator for Box<I>
where
    I: IterIndicator,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.as_mut().next()
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.as_ref().offset()
    }
}

pub struct ClosureIndicator<S, V, F>
// where
//     F: FnMut(S) -> MaybeValue<V>,
{
    closure: F,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V>,
}

impl<S, V, F> Indicator for ClosureIndicator<S, V, F>
where
    S: Sequence,
    V: std::fmt::Debug,
    F: FnMut(S) -> MaybeValue<V>,
{
    type Seq = S;
    type Val = V;
}

impl<S, V, F> ClosureIndicator<S, V, F> {
    pub fn new(closure: F) -> Self {
        Self {
            closure: closure,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

impl<S, V, F> FuncIndicator for ClosureIndicator<S, V, F>
where
    S: Sequence,
    V: std::fmt::Debug,
    F: Fn(S) -> MaybeValue<V>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        (self.closure)(seq)
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

#[cfg(feature = "ffi")]
#[macro_use]
pub mod ffi {
    use super::*;
    use crate::ffi::*;
    use crate::granularity::ffi::*;
    use crate::time::ffi::*;
    use std::ops::Deref;

    #[repr(C)]
    pub struct CMaybeFixed<T> {
        is_fixed: c_char,
        value: T,
    }

    impl<T> CMaybeFixed<T>
    where
        T: Default,
    {
        pub fn not_fixed() -> Self {
            Self {
                is_fixed: 0,
                value: Default::default(),
            }
        }

        pub fn fixed(value: T) -> Self {
            Self {
                is_fixed: 1,
                value: value,
            }
        }

        pub fn from_option(value: Option<T>) -> Self {
            match value {
                Some(value) => Self::fixed(value),
                None => Self::not_fixed(),
            }
        }
    }

    impl<V> Default for CMaybeFixed<V>
    where
        V: Default,
    {
        fn default() -> Self {
            Self::not_fixed()
        }
    }

    impl<V> From<MaybeFixed<V>> for CMaybeFixed<V>
    where
        V: Default,
    {
        fn from(v: MaybeFixed<V>) -> Self {
            match v {
                MaybeFixed::Fixed(v) => CMaybeFixed::fixed(v),
                MaybeFixed::NotFixed => CMaybeFixed::not_fixed(),
            }
        }
    }

    #[repr(C)]
    pub struct CMaybeInRange<T> {
        is_in_range: c_char,
        value: T,
    }

    impl<T> CMaybeInRange<T>
    where
        T: Default,
    {
        pub fn out_of_range() -> Self {
            Self {
                is_in_range: 0,
                value: Default::default(),
            }
        }

        pub fn in_range(value: T) -> Self {
            Self {
                is_in_range: 1,
                value: value,
            }
        }

        pub fn from_option(value: Option<T>) -> Self {
            match value {
                Some(value) => Self::in_range(value),
                None => Self::out_of_range(),
            }
        }
    }

    impl<V> Default for CMaybeInRange<V>
    where
        V: Default,
    {
        fn default() -> Self {
            Self::out_of_range()
        }
    }

    impl<V> From<MaybeInRange<V>> for CMaybeInRange<V>
    where
        V: Default,
    {
        fn from(v: MaybeInRange<V>) -> Self {
            match v {
                MaybeInRange::InRange(v) => CMaybeInRange::in_range(v),
                MaybeInRange::OutOfRange => CMaybeInRange::out_of_range(),
            }
        }
    }

    #[derive(Clone)]
    pub struct FuncIndicatorPtr<S, V>(pub Rc<RefCell<dyn FuncIndicator<Seq = S, Val = V>>>);

    impl<S, V> Indicator for FuncIndicatorPtr<S, V>
    where
        V: std::fmt::Debug,
        S: Sequence,
    {
        type Seq = S;
        type Val = V;
    }

    impl<S, V> FuncIndicator for FuncIndicatorPtr<S, V>
    where
        V: std::fmt::Debug,
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

    pub type CMaybeValue<T> = CMaybeFixed<CMaybeInRange<T>>;

    impl<T> From<MaybeValue<T>> for CMaybeValue<T>
    where
        T: Default,
    {
        fn from(v: MaybeValue<T>) -> Self {
            match v {
                MaybeFixed::Fixed(MaybeInRange::InRange(v)) => {
                    CMaybeFixed::fixed(CMaybeInRange::in_range(v))
                }
                MaybeFixed::Fixed(MaybeInRange::OutOfRange) => {
                    CMaybeFixed::fixed(CMaybeInRange::out_of_range())
                }
                MaybeFixed::NotFixed => CMaybeFixed::not_fixed(),
            }
        }
    }

    pub unsafe fn value<S, CS, V, CV>(ptr: *mut FuncIndicatorPtr<S, V>, seq: CS) -> CMaybeValue<CV>
    where
        CS: Into<S>,
        CV: From<V> + Default,
    {
        if ptr.is_null() {
            panic!("pointer is null");
        }

        let ptr = &*ptr;
        CMaybeValue::from(ptr.borrow().value(seq.into()).map2(CV::from))
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
        // ?????? Box ???????????????????????????
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
    define_value!(TickId, i64, f64, f64, indicator_value_tick_id_f64);
    define_value!(TickId, i64, GTime<Var>, CTime, indicator_value_tick_id_time);

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

    // impl Into<i32> for ZoneId {
    //     fn into(self) -> i32 {
    //         self.0
    //     }
    // }

    // impl From<i32> for ZoneId {
    //     fn from(i: i32) -> Self {
    //         ZoneId(i)
    //     }
    // }

    impl From<ZoneId> for i32 {
        fn from(z: ZoneId) -> Self {
            z.0
        }
    }

    use crate::strategy::busena::zone::*;
    define_value!(TickId, i64, ZoneId, i32, indicator_value_tick_id_zone_id);

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
pub mod ema;
pub mod envelope;
pub mod ordering;
pub mod rolling;
pub mod slope;
pub mod sma;
pub mod storage;
pub mod stream;
pub mod tick;
pub mod trade;
pub mod transaction;
pub mod vec;
// pub mod trailing_stop;
