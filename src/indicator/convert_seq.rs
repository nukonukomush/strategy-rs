use super::*;

pub struct Consume<S1, S2, V1, V2, I, F> {
    offset: S2,
    source: I,
    func: F,
    p1: std::marker::PhantomData<S1>,
    p2: std::marker::PhantomData<V1>,
    p3: std::marker::PhantomData<V2>,
}

impl<S1, S2, V1, V2, I, F> Consume<S1, S2, V1, V2, I, F>
where
    S1: Sequence,
    S2: Sequence,
    I: IterIndicator<S1, V1>,
    // F: FnMut(&mut I) -> MaybeValue<V2>,
    F: FnMut(Internal<S1, V1, I>) -> MaybeValue<V2>,
{
    pub fn new(offset: S2, source: I, func: F) -> Self {
        Self {
            offset: offset,
            source: source,
            func: func,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
            p3: std::marker::PhantomData,
        }
    }
}

impl<S1, S2, V1, V2, I, F> Indicator<S2, V2> for Consume<S1, S2, V1, V2, I, F> {}

impl<S1, S2, V1, V2, I, F> IterIndicator<S2, V2> for Consume<S1, S2, V1, V2, I, F>
where
    S1: Sequence,
    S2: Sequence,
    I: IterIndicator<S1, V1>,
    // F: FnMut(&mut I) -> MaybeValue<V2>,
    F: FnMut(Internal<S1, V1, I>) -> MaybeValue<V2>,
{
    fn next(&mut self) -> MaybeValue<V2> {
        self.offset = self.offset + 1;
        // (self.func)(&mut self.source)
        (self.func)(Internal::new(&mut self.source))
    }

    fn offset(&self) -> S2 {
        self.offset
    }
}

pub struct Internal<'a, S, V, I> {
    source: &'a mut I,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V>,
}

impl<'a, S, V, I> Internal<'a, S, V, I> {
    pub fn new(source: &'a mut I) -> Self {
        Self {
            source: source,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

impl<'a, S, V, I> Indicator<S, V> for Internal<'a, S, V, I> {}
impl<'a, S, V, I> IterIndicator<S, V> for Internal<'a, S, V, I>
where
    S: Sequence,
    I: IterIndicator<S, V>,
{
    fn next(&mut self) -> MaybeValue<V> {
        self.source.next()
    }

    fn offset(&self) -> S {
        self.source.offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::indicator::*;
    use crate::vec::*;
    use MaybeValue::*;

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
            MaybeValue::Value(v)
        });

        assert_eq!(consume.next(), Value(6.0));
        source.borrow_mut().add(4.0);
        assert_eq!(consume.next(), Value(4.0));
        source.borrow_mut().add(5.0);
        source.borrow_mut().add(2.0);
        assert_eq!(consume.next(), Value(7.0));
        assert_eq!(consume.next(), Value(0.0));
    }

}
