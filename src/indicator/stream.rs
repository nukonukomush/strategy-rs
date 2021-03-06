use super::*;
use crate::seq::*;
use crate::time::*;
use log::*;
use MaybeFixed::*;
use MaybeInRange::*;
// use std::fmt::Debug;

pub struct Map<I, F> {
    source: I,
    func: F,
}

impl<I, F> Map<I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
        }
    }
}

impl<V, I, F> Indicator for Map<I, F>
where
    V: std::fmt::Debug,
    I: Indicator,
    F: FnMut(I::Val) -> V,
{
    type Seq = I::Seq;
    type Val = V;
}

impl<V, I, F> FuncIndicator for Map<I, F>
where
    V: std::fmt::Debug,
    I: FuncIndicator,
    F: Fn(I::Val) -> V,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.source.value(seq).map(|v| v.map(|v| (self.func)(v)))
    }
}

impl<V, I, F> IterIndicator for Map<I, F>
where
    V: std::fmt::Debug,
    I: IterIndicator,
    F: FnMut(I::Val) -> V,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.source.next().map(|v| v.map(|v| (self.func)(v)))
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

pub struct Then<I, F> {
    source: I,
    func: F,
}

impl<I, F> Then<I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
        }
    }
}

impl<V, I, F> Indicator for Then<I, F>
where
    V: std::fmt::Debug,
    I: Indicator,
    F: FnMut(MaybeValue<I::Val>) -> MaybeValue<V>,
{
    type Seq = I::Seq;
    type Val = V;
}

impl<V, I, F> FuncIndicator for Then<I, F>
where
    V: std::fmt::Debug,
    I: FuncIndicator,
    F: Fn(MaybeValue<I::Val>) -> MaybeValue<V>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        (self.func)(self.source.value(seq))
    }
}

impl<V, I, F> IterIndicator for Then<I, F>
where
    V: std::fmt::Debug,
    I: IterIndicator,
    F: FnMut(MaybeValue<I::Val>) -> MaybeValue<V>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        (self.func)(self.source.next())
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

pub struct AndThen<I, F> {
    source: I,
    func: F,
}

impl<I, F> AndThen<I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
        }
    }
}

impl<V, I, F> Indicator for AndThen<I, F>
where
    V: std::fmt::Debug,
    I: Indicator,
    F: FnMut(I::Val) -> MaybeValue<V>,
{
    type Seq = I::Seq;
    type Val = V;
}

impl<V, I, F> FuncIndicator for AndThen<I, F>
where
    V: std::fmt::Debug,
    I: FuncIndicator,
    F: Fn(I::Val) -> MaybeValue<V>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        (self.func)(try_value!(self.source.value(seq)))
    }
}

impl<V, I, F> IterIndicator for AndThen<I, F>
where
    V: std::fmt::Debug,
    I: IterIndicator,
    F: FnMut(I::Val) -> MaybeValue<V>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        (self.func)(try_value!(self.source.next()))
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

pub struct WhenNotFixed<I, F> {
    source: I,
    func: F,
}

impl<I, F> WhenNotFixed<I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
        }
    }
}

impl<I, F> Indicator for WhenNotFixed<I, F>
where
    I: Indicator,
    F: FnMut() -> MaybeValue<I::Val>,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I, F> FuncIndicator for WhenNotFixed<I, F>
where
    I: FuncIndicator,
    F: Fn() -> MaybeValue<I::Val>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        match self.source.value(seq) {
            NotFixed => (self.func)(),
            other => other,
        }
    }
}

impl<I, F> IterIndicator for WhenNotFixed<I, F>
where
    I: IterIndicator,
    F: FnMut() -> MaybeValue<I::Val>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        match self.source.next() {
            NotFixed => (self.func)(),
            other => other,
        }
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

pub struct WhenOutOfRange<I, F> {
    source: I,
    func: F,
}

impl<I, F> WhenOutOfRange<I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
        }
    }
}

impl<I, F> Indicator for WhenOutOfRange<I, F>
where
    I: Indicator,
    F: FnMut() -> MaybeValue<I::Val>,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I, F> FuncIndicator for WhenOutOfRange<I, F>
where
    I: FuncIndicator,
    F: Fn() -> MaybeValue<I::Val>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        match self.source.value(seq) {
            Fixed(OutOfRange) => (self.func)(),
            other => other,
        }
    }
}

impl<I, F> IterIndicator for WhenOutOfRange<I, F>
where
    I: IterIndicator,
    F: FnMut() -> MaybeValue<I::Val>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        match self.source.next() {
            Fixed(OutOfRange) => (self.func)(),
            other => other,
        }
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

pub struct FuncZip<I1, I2> {
    source_1: I1,
    source_2: I2,
}

impl<I1, I2> FuncZip<I1, I2> {
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
        }
    }
}

impl<I1, I2> Indicator for FuncZip<I1, I2>
where
    I1: Indicator,
    I2: Indicator<Seq = I1::Seq>,
{
    type Seq = I1::Seq;
    type Val = (I1::Val, I2::Val);
}

impl<I1, I2> FuncIndicator for FuncZip<I1, I2>
where
    I1: FuncIndicator,
    I2: FuncIndicator<Seq = I1::Seq>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let v1 = try_value!(self.source_1.value(seq));
        let v2 = try_value!(self.source_2.value(seq));
        Fixed(InRange((v1, v2)))
    }
}

pub struct IterZip<V1, V2, I1, I2> {
    source_1: I1,
    source_2: I2,
    value_1: MaybeFixed<MaybeInRange<V1>>,
    value_2: MaybeFixed<MaybeInRange<V2>>,
}

impl<V1, V2, I1, I2> IterZip<V1, V2, I1, I2> {
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            value_1: NotFixed,
            value_2: NotFixed,
        }
    }
}

impl<V1, V2, I1, I2> Indicator for IterZip<V1, V2, I1, I2>
where
    V1: std::fmt::Debug,
    V2: std::fmt::Debug,
    I1: Indicator<Val = V1>,
    I2: Indicator<Seq = I1::Seq, Val = V2>,
{
    type Seq = I1::Seq;
    type Val = (I1::Val, I2::Val);
}

impl<V1, V2, I1, I2> IterIndicator for IterZip<V1, V2, I1, I2>
where
    V1: Clone + std::fmt::Debug,
    V2: Clone + std::fmt::Debug,
    I1: IterIndicator<Val = V1>,
    I2: IterIndicator<Seq = I1::Seq, Val = V2>,
{
    // TODO: not use clone
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        // trace!("[next start] offset: {:?}", self.offset());
        if self.value_1.is_not_fixed() {
            self.value_1 = self.source_1.next();
            trace!("value_1 = {:?}", self.value_1);
        }
        if self.value_2.is_not_fixed() {
            self.value_2 = self.source_2.next();
            trace!("value_2 = {:?}", self.value_2);
        }
        match (self.value_1.clone(), self.value_2.clone()) {
            (Fixed(v1), Fixed(v2)) => {
                self.value_1 = NotFixed;
                self.value_2 = NotFixed;
                Fixed(v1.zip(v2))
            }
            _ => NotFixed,
        }
        // if self.value_1.is_fixed() && self.value_2.is_fixed() {
        //     self.value_1
        //     self.value_1 = NotFixed;
        //     self.value_2 = NotFixed;
        // }
        // let v1 = try_fixed!(&self.value_1).clone();
        // let v2 = try_fixed!(&self.value_2).clone();
        // self.value_1 = NotFixed;
        // self.value_2 = NotFixed;
        // let ret = Fixed(InRange((v1, v2)));
        // debug!(
        //     "return next value: value = {:?}, offset = {:?}",
        //     &ret,
        //     self.offset()
        // );
        // ret
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source_1.offset()
    }
}

pub struct StdIter<I> {
    source: I,
}

impl<I> StdIter<I> {
    pub fn new(source: I) -> Self {
        Self { source: source }
    }
}

impl<I> Iterator for StdIter<I>
where
    I: IterIndicator,
{
    type Item = I::Val;
    #[inline]
    fn next(&mut self) -> Option<I::Val> {
        match self.source.next() {
            Fixed(InRange(v)) => Some(v),
            _ => None,
        }
    }
}

pub struct FuncIter<S, I> {
    source: I,
    offset: S,
}

impl<S, I> FuncIter<S, I> {
    pub fn new(source: I, offset: S) -> Self {
        Self {
            source: source,
            offset: offset,
        }
    }
}

impl<S, I> Indicator for FuncIter<S, I>
where
    S: Sequence,
    I: Indicator<Seq = S>,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<S, I> IterIndicator for FuncIter<S, I>
where
    S: Sequence,
    I: FuncIndicator<Seq = S>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        let v = try_fixed!(self.source.value(self.offset));
        self.offset = self.offset + 1;
        Fixed(v)
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.offset
    }
}

// pub struct IterVec<S, V, I> {
//     source: RefCell<I>,
//     vec: RefCell<vec::VecIndicator<S, V>>,
// }

// impl<S, V, I> IterVec<S, V, I>
// where
//     // S: Granularity + Copy + Ord,
//     S: Sequence,
//     I: IterIndicator<S, V>,
// {
//     // TODO: initial capacity
//     pub fn new(source: I) -> Self {
//         Self {
//             vec: RefCell::new(vec::VecIndicator::new(source.offset(), Vec::new())),
//             source: RefCell::new(source),
//         }
//     }

//     fn update_to(&self, seq: S) {
//         let mut source = self.source.borrow_mut();
//         while source.offset() <= seq {
//             match source.next() {
//                 MaybeValue::Value(v) => self.vec.borrow_mut().add(v),
//                 MaybeValue::OutOfRange => return,
//             }
//         }
//     }
// }

// impl<S, V, I> Indicator<S, V> for IterVec<S, V, I>
// where
//     I: Indicator<S, V>,
// {
//     // fn granularity(&self) -> S {
//     //     self.source.granularity()
//     // }
// }

// impl<S, V, I> FuncIndicator<S, V> for IterVec<S, V, I>
// where
//     // S: Granularity + Copy + Ord,
//     S: Sequence,
//     V: Clone,
//     I: IterIndicator<S, V>,
// {
//     fn value(&self, seq: S) -> MaybeValue<V> {
//         self.update_to(seq);
//         self.vec.value(seq)
//     }
// }

// pub struct IterStorage<S, V, I> {
//     source: I,
//     storage: storage::Storage<S, V>,
// }

// impl<S, V, I> IterStorage<S, V, I>
// where
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     I: IterIndicator<S, V>,
// {
//     // TODO: initial capacity
//     pub fn new(source: I) -> Self {
//         Self {
//             storage: storage::Storage::new(source.offset()),
//             source: source,
//         }
//     }
// }

// impl<S, V, I> IterStorage<S, V, I>
// where
//     Self: IterIndicator<S, V>,
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     V: Clone,
//     I: IterIndicator<S, V>,
// {
//     pub fn update_to(&mut self, seq: S) {
//         while self.source.offset() <= seq {
//             self.next();
//         }
//     }

//     pub fn into_consumer(self) -> IterConsumerStorage<S, V, I> {
//         IterConsumerStorage::new(self)
//     }
// }

// impl<S, V, I> Indicator<S, V> for IterStorage<S, V, I>
// where
//     I: Indicator<S, V>,
// {
//     // fn granularity(&self) -> S {
//     //     self.source.granularity()
//     // }
// }

// impl<S, V, I> FuncIndicator<S, V> for IterStorage<S, V, I>
// where
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     V: Clone,
//     I: IterIndicator<S, V>,
// {
//     fn value(&self, seq: S) -> MaybeValue<V> {
//         self.storage.value(seq).map(|v| v.unwrap())
//     }
// }

// impl<S, V, I> IterIndicator<S, V> for IterStorage<S, V, I>
// where
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     V: Clone,
//     I: IterIndicator<S, V>,
// {
//     fn next(&mut self) -> MaybeValue<V> {
//         let seq = self.source.offset();
//         let v = try_value!(self.source.next());
//         self.storage.add(seq, v.clone());
//         MaybeValue::Value(v)
//     }

//     fn offset(&self) -> S {
//         self.source.offset()
//     }
// }

// use std::cell::RefCell;
// pub struct IterConsumerStorage<S, V, I> {
//     // TODO: generics
//     source: RefCell<IterStorage<S, V, I>>,
// }

// impl<S, V, I> IterConsumerStorage<S, V, I>
// where
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     I: IterIndicator<S, V>,
// {
//     // TODO: initial capacity
//     pub fn new(source: IterStorage<S, V, I>) -> Self {
//         Self {
//             source: RefCell::new(source),
//         }
//     }
// }

// impl<S, V, I> Indicator<S, V> for IterConsumerStorage<S, V, I>
// where
//     I: Indicator<S, V>,
// {
//     // fn granularity(&self) -> S {
//     //     self.source.granularity()
//     // }
// }

// impl<S, V, I> FuncIndicator<S, V> for IterConsumerStorage<S, V, I>
// where
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     V: Clone,
//     I: IterIndicator<S, V>,
// {
//     fn value(&self, seq: S) -> MaybeValue<V> {
//         self.source.borrow_mut().update_to(seq);
//         self.source.value(seq)
//     }
// }

// impl<S, V, I> IterIndicator<S, V> for IterConsumerStorage<S, V, I>
// where
//     // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
//     S: Sequence,
//     V: Clone,
//     I: IterIndicator<S, V>,
// {
//     fn next(&mut self) -> MaybeValue<V> {
//         self.source.borrow_mut().next()
//     }

//     fn offset(&self) -> S {
//         self.source.borrow().offset()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_zip() {
        let offset = Time::<S5>::new(0);
        let source_1 = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let source_2 = vec![0, -1, 0, 1, 0_i32];
        let expect = vec![
            Fixed(InRange(0.0)),
            Fixed(InRange(2.0)),
            Fixed(InRange(0.0)),
            Fixed(InRange(4.0)),
            Fixed(InRange(0.0)),
        ];
        let vec_1 = VecIndicator::new(offset, source_1);
        let vec_2 = VecIndicator::new(offset, source_2);
        let func = vec_1.zip(vec_2).map(|(v1, v2)| v1 * v2.abs() as f64);

        let result = (0..5).map(|i| func.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_iter() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![1.0, 3.0, 6.0, 10.0, 15.0];
        let mut sum = 0.0;

        let vec = VecIndicator::new(offset, source.clone());
        let iter = IterIndicator::map(vec.into_iter(offset), move |v| {
            sum += v;
            sum
        });
        let result = iter.into_std().collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    // #[test]
    // fn test_via_iter() {
    //     let offset = Time::<S5>::new(0);
    //     let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    //     let mut sum = 0.0;
    //     let count = Rc::new(RefCell::new(0));
    //     let count_move = count.clone();

    //     let vec = VecIndicator::new(offset, source.clone());
    //     let iter = IterVec::new(IterIndicator::map(vec.into_iter(offset), move |v| {
    //         *count_move.borrow_mut() += 1;
    //         sum += v;
    //         sum
    //     }));
    //     assert_eq!(iter.value(offset + 4), MaybeValue::Value(15.0));
    //     assert_eq!(iter.value(offset + 5), MaybeValue::OutOfRange);
    //     assert_eq!(iter.value(offset + 3), MaybeValue::Value(10.0));
    //     assert_eq!(*count.borrow(), 5);
    // }
}
