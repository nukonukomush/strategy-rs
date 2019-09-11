use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Ordering<I1, I2> {
    source_1: I1,
    source_2: I2,
}

impl<I1, I2> Ordering<I1, I2>
where
    I1: Indicator,
    I2: Indicator,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
        }
    }
}

impl<I1, I2> Indicator for Ordering<I1, I2>
where
    I1: Indicator,
    I2: Indicator<Seq = I1::Seq>,
{
    type Seq = I1::Seq;
    type Val = std::cmp::Ordering;
}

impl<V, I1, I2> FuncIndicator for Ordering<I1, I2>
where
    V: PartialOrd + std::fmt::Debug,
    I1: FuncIndicator<Val = V>,
    I2: FuncIndicator<Seq = I1::Seq, Val = V>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let v1 = try_value!(self.source_1.value(seq));
        let v2 = try_value!(self.source_2.value(seq));
        let ord = v1.partial_cmp(&v2).unwrap();
        Fixed(InRange(ord))
    }
}
