use super::*;
use crate::granularity::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct ConvertWithNone<S, I> {
    source: I,
    phantom: std::marker::PhantomData<S>,
}

impl<S, I> ConvertWithNone<S, I> {
    pub fn new(source: I) -> Self {
        Self {
            source: source,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<S2, I> Indicator for ConvertWithNone<S2, I>
where
    S2: Sequence,
    I: Indicator,
{
    type Seq = S2;
    type Val = Option<I::Val>;
}

impl<G1, G2, I> FuncIndicator for ConvertWithNone<Time<G2>, I>
where
    G1: StaticGranularity,
    G2: StaticGranularity,
    I: FuncIndicator<Seq = Time<G1>>,
{
    fn value(&self, time: Self::Seq) -> MaybeValue<Self::Val> {
        match time.try_into() {
            Ok(time) => self.source.value(time).map(|v| v.map(|v| Some(v))),
            Err(_) => Fixed(InRange(None)),
        }
    }
}

// impl<G1, G2, V, I> Indicator<G2, Option<V>> for ConvertWithNone<G1, G2, I>
// where
//     G1: Granularity + Copy,
//     G2: Granularity + Copy,
//     I: Indicator<G1, V>,
// {
//     fn granularity(&self) -> G2 {
//         self.granularity
//     }
// }

// impl<G1, G2, V, I> FuncIndicator<G2, Option<V>> for ConvertWithNone<G1, G2, I>
// where
//     G1: Granularity + Copy,
//     G2: Granularity + Copy,
//     I: FuncIndicator<G1, V>,
// {
//     fn value(&self, time: Time<G2>) -> MaybeValue<Option<V>> {
//         match time.try_into(self.source.granularity()) {
//             Ok(time) => self.source.value(time).map(|v| Some(v)),
//             Err(_) => MaybeValue::Value(None),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;

    #[test]
    fn test_conv_s5_to_s10() {
        let offset_s5 = Time::<S5>::new(0);
        let offset_s10 = Time::<S10>::new(0);
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let expect = vec![
            Fixed(InRange(Some(1.0))),
            Fixed(InRange(Some(3.0))),
            Fixed(InRange(Some(5.0))),
        ];
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
            Fixed(InRange(Some(1.0))),
            Fixed(InRange(None)),
            Fixed(InRange(Some(2.0))),
            Fixed(InRange(None)),
            Fixed(InRange(Some(3.0))),
            Fixed(InRange(None)),
            Fixed(InRange(Some(4.0))),
            Fixed(InRange(None)),
            Fixed(InRange(Some(5.0))),
        ];
        let conv = ConvertWithNone::new(VecIndicator::new(offset_s10, source.clone()));

        let result = (0..9)
            .map(|i| conv.value(offset_s5 + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
