use super::trade::*;
use super::*;
use crate::transaction::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct ProfitLoss<I> {
    trade_histories: I,
}

impl<I> ProfitLoss<I> {
    pub fn new(source: I) -> Self {
        Self {
            trade_histories: source,
        }
    }
}

impl<I> Indicator for ProfitLoss<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = f64;
}

impl<I> FuncIndicator for ProfitLoss<I>
where
    I: FuncIndicator<Val = Option<Trade>>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let pl = match try_value!(self.trade_histories.value(seq)) {
            Some(trade) => {
                let distance = match trade.long_or_short {
                    LongOrShort::Long => trade.close_price - trade.open_price,
                    LongOrShort::Short => trade.open_price - trade.close_price,
                };
                distance * trade.unit as f64
            }
            None => 0.0,
        };
        Fixed(InRange(pl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;
    use approx::assert_relative_eq;
    use LongOrShort::*;

    #[test]
    fn test_pl() {
        let offset = TransactionId(10);
        let time = Time::<S5>::new(0);
        let source = VecIndicator::new(
            offset,
            vec![
                None,
                None,
                Some(Trade {
                    unit: 100,
                    long_or_short: Long,
                    open_time: (time + 0).into(),
                    close_time: (time + 5).into(),
                    open_price: 1.234,
                    close_price: 1.5,
                }),
                Some(Trade {
                    unit: 100,
                    long_or_short: Short,
                    open_time: (time + 3).into(),
                    close_time: (time + 9).into(),
                    open_price: 1.4,
                    close_price: 1.1,
                }),
            ],
        );
        let expect = vec![
            Fixed(InRange(0.0)),
            Fixed(InRange(0.0)),
            Fixed(InRange(26.6)),
            Fixed(InRange(30.0)),
        ];
        let pl = ProfitLoss::new(source);

        let result = (0..4).map(|i| pl.value(offset + i)).collect::<Vec<_>>();
        assert_relative_eq!(
            result.as_slice(),
            expect.as_slice(),
            max_relative = 0.0000001
        );

        let mut sum = 0.0;
        let balance = pl.into_iter(offset).map(move |v| {
            sum += v;
            sum
        });
        let balance_result = balance.into_std().collect::<Vec<_>>();
        let balance_expect = vec![0.0, 0.0, 26.6, 56.6];
        assert_relative_eq!(
            balance_result.as_slice(),
            balance_expect.as_slice(),
            max_relative = 0.0000001
        );
    }
}
