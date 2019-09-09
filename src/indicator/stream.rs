use super::*;
use crate::seq::*;
use crate::time::*;

pub struct Map<S, V1, V2, I, F> {
    source: I,
    func: F,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V1>,
    p3: std::marker::PhantomData<V2>,
}

impl<S, V1, V2, I, F> Map<S, V1, V2, I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
            p3: std::marker::PhantomData,
        }
    }
}

impl<S, V1, V2, I, F> Indicator<S, V2> for Map<S, V1, V2, I, F>
where
    I: Indicator<S, V1>,
    F: FnMut(V1) -> V2,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, V1, V2, I, F> FuncIndicator<S, V2> for Map<S, V1, V2, I, F>
where
    I: FuncIndicator<S, V1>,
    F: Fn(V1) -> V2,
{
    fn value(&self, seq: S) -> MaybeValue<V2> {
        self.source.value(seq).map(|v| (self.func)(v))
    }
}

impl<S, V1, V2, I, F> IterIndicator<S, V2> for Map<S, V1, V2, I, F>
where
    I: IterIndicator<S, V1>,
    F: FnMut(V1) -> V2,
{
    fn next(&mut self) -> MaybeValue<V2> {
        self.source.next().map(|v| (self.func)(v))
    }

    fn offset(&self) -> S {
        self.source.offset()
    }
}

// TODO: then は使わないかも？けす
// pub struct Then<S, V1, V2, I, F> {
//     source: I,
//     func: F,
//     p1: std::marker::PhantomData<S>,
//     p2: std::marker::PhantomData<V1>,
//     p3: std::marker::PhantomData<V2>,
// }

// impl<S, V1, V2, I, F> Then<S, V1, V2, I, F> {
//     pub fn new(source: I, func: F) -> Self {
//         Self {
//             source: source,
//             func: func,
//             p1: std::marker::PhantomData,
//             p2: std::marker::PhantomData,
//             p3: std::marker::PhantomData,
//         }
//     }
// }

// impl<S, V1, V2, I, F> Indicator<S, V2> for Then<S, V1, V2, I, F>
// where
//     I: Indicator<S, V1>,
//     F: Fn(MaybeValue<V1>) -> MaybeValue<V2>,
// {
//     fn granularity(&self) -> S {
//         self.source.granularity()
//     }
// }

// impl<S, V1, V2, I, F> FuncIndicator<S, V2> for Then<S, V1, V2, I, F>
// where
//     I: FuncIndicator<S, V1>,
//     F: Fn(MaybeValue<V1>) -> MaybeValue<V2>,
// {
//     fn value(&self, seq: S) -> MaybeValue<V2> {
//         (self.func)(self.source.value(seq))
//     }
// }

pub struct Zip<S, V1, V2, I1, I2> {
    source_1: I1,
    source_2: I2,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V1>,
    p3: std::marker::PhantomData<V2>,
}

impl<S, V1, V2, I1, I2> Zip<S, V1, V2, I1, I2> {
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
            p3: std::marker::PhantomData,
        }
    }
}

impl<S, V1, V2, I1, I2> Indicator<S, (V1, V2)> for Zip<S, V1, V2, I1, I2>
where
    I1: Indicator<S, V1>,
    I2: Indicator<S, V2>,
{
    // fn granularity(&self) -> S {
    //     self.source_1.granularity()
    // }
}

impl<S, V1, V2, I1, I2> FuncIndicator<S, (V1, V2)> for Zip<S, V1, V2, I1, I2>
where
    // S: Granularity + Copy,
    S: Sequence,
    I1: FuncIndicator<S, V1>,
    I2: FuncIndicator<S, V2>,
{
    fn value(&self, seq: S) -> MaybeValue<(V1, V2)> {
        let v1 = try_value!(self.source_1.value(seq));
        let v2 = try_value!(self.source_2.value(seq));
        MaybeValue::Value((v1, v2))
    }
}

impl<S, V1, V2, I1, I2> IterIndicator<S, (V1, V2)> for Zip<S, V1, V2, I1, I2>
where
    I1: IterIndicator<S, V1>,
    I2: IterIndicator<S, V2>,
{
    // FIXME: v1 => ok, v2 => ng のときにバグるので、v1 を持っておくようにする
    fn next(&mut self) -> MaybeValue<(V1, V2)> {
        let v1 = try_value!(self.source_1.next());
        let v2 = try_value!(self.source_2.next());
        MaybeValue::Value((v1, v2))
    }

    fn offset(&self) -> S {
        self.source_1.offset()
    }
}

pub struct StdIter<S, V, I> {
    source: I,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V>,
}

impl<S, V, I> StdIter<S, V, I> {
    pub fn new(source: I) -> Self {
        Self {
            source: source,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

impl<S, V, I> Iterator for StdIter<S, V, I>
where
    I: IterIndicator<S, V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        self.source.next().into()
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

impl<S, V, I> Indicator<S, V> for FuncIter<S, I>
where
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

// impl<S, V, I> FuncIndicator<S, V> for FuncIter<S, I>
// where
//     // S: Granularity,
//     S: Sequence,
//     I: FuncIndicator<S, V>,
// {
//     fn value(&self, seq: S) -> MaybeValue<V> {
//         self.source.value(seq)
//     }
// }

impl<S, V, I> IterIndicator<S, V> for FuncIter<S, I>
where
    // S: Granularity + Copy,
    S: Sequence,
    I: FuncIndicator<S, V>,
{
    fn next(&mut self) -> MaybeValue<V> {
        let v = try_value!(self.source.value(self.offset));
        self.offset = self.offset + 1;
        MaybeValue::Value(v)
    }

    fn offset(&self) -> S {
        self.offset
    }
}

pub struct IterVec<S, V, I> {
    source: RefCell<I>,
    vec: RefCell<vec::VecIndicator<S, V>>,
}

impl<S, V, I> IterVec<S, V, I>
where
    // S: Granularity + Copy + Ord,
    S: Sequence,
    I: IterIndicator<S, V>,
{
    // TODO: initial capacity
    pub fn new(source: I) -> Self {
        Self {
            vec: RefCell::new(vec::VecIndicator::new(source.offset(), Vec::new())),
            source: RefCell::new(source),
        }
    }

    fn update_to(&self, seq: S) {
        let mut source = self.source.borrow_mut();
        while source.offset() <= seq {
            match source.next() {
                MaybeValue::Value(v) => self.vec.borrow_mut().add(v),
                MaybeValue::OutOfRange => return,
            }
        }
    }
}

impl<S, V, I> Indicator<S, V> for IterVec<S, V, I>
where
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, V, I> FuncIndicator<S, V> for IterVec<S, V, I>
where
    // S: Granularity + Copy + Ord,
    S: Sequence,
    V: Clone,
    I: IterIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        self.update_to(seq);
        self.vec.value(seq)
    }
}

pub struct IterStorage<S, V, I> {
    source: I,
    storage: storage::Storage<S, V>,
}

impl<S, V, I> IterStorage<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    I: IterIndicator<S, V>,
{
    // TODO: initial capacity
    pub fn new(source: I) -> Self {
        Self {
            storage: storage::Storage::new(source.offset()),
            source: source,
        }
    }
}

impl<S, V, I> IterStorage<S, V, I>
where
    Self: IterIndicator<S, V>,
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    V: Clone,
    I: IterIndicator<S, V>,
{
    pub fn update_to(&mut self, seq: S) {
        while self.source.offset() <= seq {
            self.next();
        }
    }

    pub fn into_consumer(self) -> IterConsumerStorage<S, V, I> {
        IterConsumerStorage::new(self)
    }
}

impl<S, V, I> Indicator<S, V> for IterStorage<S, V, I>
where
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, V, I> FuncIndicator<S, V> for IterStorage<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    V: Clone,
    I: IterIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        self.storage.value(seq).map(|v| v.unwrap())
    }
}

impl<S, V, I> IterIndicator<S, V> for IterStorage<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    V: Clone,
    I: IterIndicator<S, V>,
{
    fn next(&mut self) -> MaybeValue<V> {
        let seq = self.source.offset();
        let v = try_value!(self.source.next());
        self.storage.add(seq, v.clone());
        MaybeValue::Value(v)
    }

    fn offset(&self) -> S {
        self.source.offset()
    }
}

use std::cell::RefCell;
pub struct IterConsumerStorage<S, V, I> {
    // TODO: generics
    source: RefCell<IterStorage<S, V, I>>,
}

impl<S, V, I> IterConsumerStorage<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    I: IterIndicator<S, V>,
{
    // TODO: initial capacity
    pub fn new(source: IterStorage<S, V, I>) -> Self {
        Self {
            source: RefCell::new(source),
        }
    }
}

impl<S, V, I> Indicator<S, V> for IterConsumerStorage<S, V, I>
where
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, V, I> FuncIndicator<S, V> for IterConsumerStorage<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    V: Clone,
    I: IterIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        self.source.borrow_mut().update_to(seq);
        self.source.value(seq)
    }
}

impl<S, V, I> IterIndicator<S, V> for IterConsumerStorage<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Ord,
    S: Sequence,
    V: Clone,
    I: IterIndicator<S, V>,
{
    fn next(&mut self) -> MaybeValue<V> {
        self.source.borrow_mut().next()
    }

    fn offset(&self) -> S {
        self.source.borrow().offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;
    use crate::granularity::*;

    #[test]
    fn test_zip() {
        let offset = Time::<S5>::new(0);
        let source_1 = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let source_2 = vec![0, -1, 0, 1, 0_i32];
        let expect = vec![Value(0.0), Value(2.0), Value(0.0), Value(4.0), Value(0.0)];
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

    #[test]
    fn test_via_iter() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut sum = 0.0;
        let count = Rc::new(RefCell::new(0));
        let count_move = count.clone();

        let vec = VecIndicator::new(offset, source.clone());
        let iter = IterVec::new(IterIndicator::map(vec.into_iter(offset), move |v| {
            *count_move.borrow_mut() += 1;
            sum += v;
            sum
        }));
        assert_eq!(iter.value(offset + 4), MaybeValue::Value(15.0));
        assert_eq!(iter.value(offset + 5), MaybeValue::OutOfRange);
        assert_eq!(iter.value(offset + 3), MaybeValue::Value(10.0));
        assert_eq!(*count.borrow(), 5);
    }
}
