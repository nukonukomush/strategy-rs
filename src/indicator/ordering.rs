use super::*;
use crate::seq::*;
use crate::*;

pub struct Ordering<S, V, I1, I2> {
    source_1: I1,
    source_2: I2,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V>,
}

impl<S, V, I1, I2> Ordering<S, V, I1, I2>
where
    S: Sequence,
    V: PartialOrd,
    I1: Indicator<S, V>,
    I2: Indicator<S, V>,
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

impl<S, V, I1, I2> Indicator<S, std::cmp::Ordering> for Ordering<S, V, I1, I2>
where
    S: Sequence,
    V: PartialOrd,
    I1: Indicator<S, V>,
    I2: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source_1.granularity()
    // }
}

impl<S, V, I1, I2> FuncIndicator<S, std::cmp::Ordering> for Ordering<S, V, I1, I2>
where
    S: Sequence,
    V: PartialOrd,
    I1: FuncIndicator<S, V>,
    I2: FuncIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<std::cmp::Ordering> {
        let v1 = try_value!(self.source_1.value(seq));
        let v2 = try_value!(self.source_2.value(seq));
        let ord = v1.partial_cmp(&v2).unwrap();
        MaybeValue::Value(ord)
    }
}
