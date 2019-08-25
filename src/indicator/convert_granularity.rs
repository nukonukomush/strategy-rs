use super::*;
use crate::time::*;

pub struct ConvertWithNone<G, I> {
    source: I,
    phantom: std::marker::PhantomData<G>,
}

impl<G, I> ConvertWithNone<G, I> {
    pub fn new(source: I) -> Self {
        Self {
            source: source,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G1, G2, V, I> Indicator<G2, V> for ConvertWithNone<G1, I>
where
    G1: Granularity,
    G2: Granularity,
    I: Indicator<G1, V>,
{
    fn value(&self, time: Time<G2>) -> Option<V> {
        match time.try_into() {
            Ok(time) => self.source.value(time),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;

    #[test]
    fn test_conv_s5_to_s10() {
        let offset_s5 = Time::<S5>::new(0);
        let offset_s10 = Time::<S10>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let expect = vec![Some(1.0), Some(3.0), Some(5.0)];
        let conv = ConvertWithNone::new(VecIndicator::new(offset_s5, source.clone()));

        let result = (0..3)
            .map(|i| conv.value(offset_s10 + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_conv_s10_to_s5() {
        let offset_s5 = Time::<S5>::new(0);
        let offset_s10 = Time::<S10>::new(0);
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
        let conv = ConvertWithNone::new(VecIndicator::new(offset_s10, source.clone()));

        let result = (0..9)
            .map(|i| conv.value(offset_s5 + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
