use super::*;
use crate::*;

pub struct Ordering<G, V, I1, I2> {
    source_1: I1,
    source_2: I2,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V>,
}

impl<G, V, I1, I2> Ordering<G, V, I1, I2>
where
    G: Granularity,
    V: PartialOrd,
    I1: Indicator<G, V>,
    I2: Indicator<G, V>,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

impl<G, V, I1, I2> Indicator<G, std::cmp::Ordering> for Ordering<G, V, I1, I2>
where
    G: Granularity + Copy,
    V: PartialOrd,
    I1: Indicator<G, V>,
    I2: Indicator<G, V>,
{
    fn granularity(&self) -> G {
        self.source_1.granularity()
    }
}

impl<G, V, I1, I2> FuncIndicator<G, std::cmp::Ordering> for Ordering<G, V, I1, I2>
where
    G: Granularity + Copy,
    V: PartialOrd,
    I1: FuncIndicator<G, V>,
    I2: FuncIndicator<G, V>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<std::cmp::Ordering> {
        let v1 = try_value!(self.source_1.value(time));
        let v2 = try_value!(self.source_2.value(time));
        let ord = v1.partial_cmp(&v2).unwrap();
        MaybeValue::Value(ord)
    }
}
