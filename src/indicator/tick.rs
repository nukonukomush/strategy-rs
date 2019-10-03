use super::*;
use chrono::prelude::*;
use log::*;
use MaybeFixed::*;
use MaybeInRange::*;

#[derive(Clone, Debug)]
pub struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open: open,
            high: high,
            low: low,
            close: close,
        }
    }
}

pub struct IntoTick<IC, IV> {
    candle: IC,
    volume: IV,
}

impl<IC, IV> IntoTick<IC, IV> {
    pub fn new(candle: IC, volume: IV) -> Self {
        Self {
            candle: candle,
            volume: volume,
        }
    }
}

impl<S, IC, IV> Indicator for IntoTick<IC, IV>
where
    S: Sequence,
    IC: Indicator<Seq = S, Val = Option<Candle>>,
    IV: Indicator<Seq = S, Val = i32>,
{
    type Seq = TickId;
    type Val = (DateTime<Utc>, f64);
}

impl<S, IC, IV> FuncIndicator for IntoTick<IC, IV>
where
    S: Sequence,
    IC: FuncIndicator<Seq = S, Val = Option<Candle>>,
    IV: FuncIndicator<Seq = S, Val = i32>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        // 多分トップダウンにやったほうがいいので、先に strategy から作る
        NotFixed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    #[ignore]
    fn test() {
        let offset = Time::<S5>::new(0);
        let src_v = vec![1, 0, 2];
        let src_c = vec![
            Some(Candle::new(1.0, 1.0, 1.0, 1.0)),
            None,
            Some(Candle::new(1.2, 2.0, 1.2, 2.0)),
        ];
        let expect = vec![
            Fixed(InRange((
                "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                1.0,
            ))),
            Fixed(InRange((
                "2019-01-01T00:00:05Z".parse::<DateTime<Utc>>().unwrap(),
                1.2,
            ))),
            Fixed(InRange((
                "2019-01-01T00:00:08Z".parse::<DateTime<Utc>>().unwrap(),
                2.0,
            ))),
        ];
        let src_v = VecIndicator::new(offset, src_v);
        let src_c = VecIndicator::new(offset, src_c);

        let into_tick = IntoTick::new(src_c, src_v);

        let result = (0..2)
            .map(|i| into_tick.value(TickId(i)))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
