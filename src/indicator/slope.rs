use super::*;

pub struct Slope<G, V, I> {
    source: I,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V>,
}

impl<G, V, I> Slope<G, V, I>
where
    I: Indicator<G, V>,
    V: std::ops::Sub,
{
    pub fn new(source: I) -> Self {
        Self {
            source: source,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

impl<G, V, I> Indicator<G, V::Output> for Slope<G, V, I>
where
    G: Granularity + Copy,
    I: Indicator<G, V>,
    V: std::ops::Sub,
{
    fn value(&self, time: Time<G>) -> Option<V::Output> {
        let cur = self.source.value(time)?;
        let prev = self.source.value(time - 1)?;
        Some(cur - prev)
    }
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;

    #[test]
    fn test_sma() {
        let offset = Time::new(0, S5);
        let source = vec![1.0, 2.0, 4.0, 8.0, 6.0];
        let expect = vec![None, Some(1.0), Some(2.0), Some(4.0), Some(-2.0)];
        let source = VecIndicator::new(offset, source);
        let slope = Slope::new(source);

        let result = (0..5).map(|i| slope.value(offset + i)).collect::<Vec<_>>();

        assert_eq!(result, expect);
    }
}
