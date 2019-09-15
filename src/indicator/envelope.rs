use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Envelope<I> {
    source: I,
    deviation: f64,
}

impl<I> Envelope<I>
where
    I: FuncIndicator,
{
    pub fn new(source: I, deviation_in_percents: f64) -> Self {
        Self {
            source: source,
            deviation: 1.0 + deviation_in_percents / 100.0,
        }
    }
}

impl<I> Indicator for Envelope<I>
where
    I: Indicator<Val = f64>,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I> FuncIndicator for Envelope<I>
where
    I: FuncIndicator<Val = f64>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.source.value(seq).map2(|v| v * self.deviation)
    }
}

impl<I> IterIndicator for Envelope<I>
where
    I: IterIndicator<Val = f64>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.source.next().map2(|v| v * self.deviation)
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}
