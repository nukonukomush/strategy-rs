use super::*;
use crate::time::*;

pub struct ConvertWithNone<G1, G2, I> {
    source: I,
    granularity: G2,
    phantom: std::marker::PhantomData<G1>,
}

impl<G1, G2, I> ConvertWithNone<G1, G2, I> {
    pub fn new(source: I, granularity: G2) -> Self {
        Self {
            source: source,
            granularity: granularity,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G1, G2, V, I> Indicator<G2, V> for ConvertWithNone<G1, G2, I>
where
    G1: Granularity + Copy,
    G2: Granularity + Copy,
    I: Indicator<G1, V>,
{
    fn value(&self, time: Time<G2>) -> Option<V> {
        match time.try_into(self.source.granularity()) {
            Ok(time) => self.source.value(time),
            Err(_) => None,
        }
    }
    fn granularity(&self) -> G2 {
        self.granularity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;

    #[test]
    fn test_conv_s5_to_s10() {
        let offset_s5 = Time::<S5>::new(0, S5);
        let offset_s10 = Time::<S10>::new(0, S10);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let expect = vec![Some(1.0), Some(3.0), Some(5.0)];
        let conv = ConvertWithNone::new(VecIndicator::new(offset_s5, source.clone()), S10);

        let result = (0..3)
            .map(|i| conv.value(offset_s10 + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_conv_s10_to_s5() {
        let offset_s5 = Time::<S5>::new(0, S5);
        let offset_s10 = Time::<S10>::new(0, S10);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let expect = vec![
            Some(1.0),
            None,
            Some(2.0),
            None,
            Some(3.0),
            None,
            Some(4.0),
            None,
            Some(5.0),
        ];
        let conv = ConvertWithNone::new(VecIndicator::new(offset_s10, source.clone()), S5);

        let result = (0..9)
            .map(|i| conv.value(offset_s5 + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
