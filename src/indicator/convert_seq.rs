use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Consume<S2, I, F> {
    offset: S2,
    source: I,
    func: F,
}

impl<S1, S2, V1, V2, I, F> Consume<S2, I, F>
where
    S1: Sequence,
    S2: Sequence,
    I: IterIndicator<Seq = S1, Val = V1>,
    F: FnMut(Internal<I>) -> MaybeValue<V2>,
{
    pub fn new(offset: S2, source: I, func: F) -> Self {
        Self {
            offset: offset,
            source: source,
            func: func,
        }
    }
}

impl<S1, S2, V1, V2, I, F> Indicator for Consume<S2, I, F>
where
    S1: Sequence,
    S2: Sequence,
    I: Indicator<Seq = S1, Val = V1>,
    F: FnMut(Internal<I>) -> MaybeValue<V2>,
{
    type Seq = S2;
    type Val = V2;
}

impl<S1, S2, V1, V2, I, F> IterIndicator for Consume<S2, I, F>
where
    S1: Sequence,
    S2: Sequence,
    I: IterIndicator<Seq = S1, Val = V1>,
    F: FnMut(Internal<I>) -> MaybeValue<V2>,
{
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.offset = self.offset + 1;
        (self.func)(Internal::new(&mut self.source))
    }

    fn offset(&self) -> Self::Seq {
        self.offset
    }
}

pub struct Internal<'a, I> {
    source: &'a mut I,
}

impl<'a, I> Internal<'a, I> {
    pub fn new(source: &'a mut I) -> Self {
        Self { source: source }
    }
}

impl<'a, I> Indicator for Internal<'a, I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<'a, I> IterIndicator for Internal<'a, I>
where
    I: IterIndicator,
{
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.source.next()
    }

    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_map() {
        let offset_1 = TransactionId(10);
        let offset_2 = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let source = VecIndicator::new(offset_1, source);
        let expect = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let consume = Consume::new(offset_2, source.into_iter(offset_1), |i| {
            i.map(|v| v * 2.0).next()
        });

        let actual = consume.into_std().collect::<Vec<_>>();
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_fold() {
        let offset_1 = TransactionId(10);
        let offset_2 = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0];
        let source = VecIndicator::new(offset_1, source).into_sync_ptr();
        let mut consume = Consume::new(offset_2, source.clone().into_iter(offset_1), |i| {
            let v = i.into_std().fold(0.0, |acc, v| acc + v);
            Fixed(InRange(v))
        });

        assert_eq!(consume.next(), Fixed(InRange(6.0)));
        source.borrow_mut().add(4.0);
        assert_eq!(consume.next(), Fixed(InRange(4.0)));
        source.borrow_mut().add(5.0);
        source.borrow_mut().add(2.0);
        assert_eq!(consume.next(), Fixed(InRange(7.0)));
        assert_eq!(consume.next(), Fixed(InRange(0.0)));
    }

}
