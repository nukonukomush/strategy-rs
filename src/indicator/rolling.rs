use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub trait Window<S, V> {
    fn sum(self) -> MaybeValue<V::Output>
    where
        V: std::ops::Add<V>;

    fn mean(self) -> MaybeValue<V::Output>
    where
        V: std::ops::Add<V>;
}

pub struct FixedSizeWindow<'a, S, I: 'a> {
    source: &'a I,
    offset: S,
    size: usize,
}

impl<'a, S, I> FixedSizeWindow<'a, S, I> {
    pub fn new(source: &'a I, offset: S, size: usize) -> Self {
        Self {
            source: source,
            offset: offset,
            size: size,
        }
    }
}

impl<'a, S, I> Window<S, f64> for FixedSizeWindow<'a, S, I>
where
    S: Sequence,
    I: FuncIndicator<Seq = S, Val = f64>,
{
    fn sum(self) -> MaybeValue<f64> {
        let mut i = self.offset;
        let mut sum = 0.0;
        let end = i + self.size as i64;
        while i < end {
            sum += try_value!(self.source.value(i));
            i = i + 1;
        }
        Fixed(InRange(sum))
    }

    fn mean(self) -> MaybeValue<f64> {
        let len = self.size as f64;
        self.sum().map2(|v| v / len)
    }
}

pub struct Rolling<I, F> {
    source: I,
    size: usize,
    func: F,
}

impl<I, V, F> Rolling<I, F>
where
    I: FuncIndicator,
    F: Fn(FixedSizeWindow<I::Seq, I>) -> MaybeValue<V>,
{
    pub fn new(source: I, size: usize, func: F) -> Self {
        Self {
            source: source,
            size: size,
            func: func,
        }
    }
}

impl<V, I, F> Indicator for Rolling<I, F>
where
    V: std::fmt::Debug,
    I: Indicator,
    F: FnMut(FixedSizeWindow<I::Seq, I>) -> MaybeValue<V>,
    // F: Fn(FixedSizeWindow<I::Seq, I>) -> MaybeValue<V>,
{
    type Seq = I::Seq;
    type Val = V;
}

impl<V, I, F> FuncIndicator for Rolling<I, F>
where
    V: std::fmt::Debug,
    I: FuncIndicator,
    F: Fn(FixedSizeWindow<I::Seq, I>) -> MaybeValue<V>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let w = FixedSizeWindow::new(&self.source, seq + 1 - self.size as i64, self.size);
        (self.func)(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_sum() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(OutOfRange),
            Fixed(InRange(6.0)),
            Fixed(InRange(9.0)),
            Fixed(InRange(12.0)),
        ];
        let source = VecIndicator::new(offset, source.clone());
        let sum = Rolling::new(source, 3, |w| w.sum());

        let result = (0..5).map(|i| sum.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    // #[test]
    // fn test_sma() {
    //     let offset = Time::<S5>::new(0);
    //     let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    //     let expect = vec![
    //         Fixed(OutOfRange),
    //         Fixed(OutOfRange),
    //         Fixed(InRange(2.0)),
    //         Fixed(InRange(3.0)),
    //         Fixed(InRange(4.0)),
    //     ];
    //     let source = VecIndicator::new(offset, source.clone());
    //     let sum = Rolling::new(source, 3, |w| w.mean());

    //     let result = (0..5).map(|i| sum.value(offset + i)).collect::<Vec<_>>();
    //     assert_eq!(result, expect);
    // }
}
