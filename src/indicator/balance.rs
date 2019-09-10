use super::trade::*;
use super::*;
use crate::indicator::*;
use crate::seq::*;
use crate::transaction::*;
use chrono::prelude::*;

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

impl<I> Indicator for ProfitLoss<I> {
    type Val = f64;
}

impl<I> FuncIndicator for ProfitLoss<I>
where
    I: FuncIndicator<Val = Option<Trade>>,
{
    fn value(&self, seq: I::Seq) -> MaybeValue<Self::Val> {
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
        MaybeValue::Value(pl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;
    use LongOrShort::*;
    use MaybeValue::*;

    use approx::assert_relative_eq;

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
        let expect = vec![Value(0.0), Value(0.0), Value(26.6), Value(30.0)];
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

// pub struct Cumulate<S, V, I> {
//     state: V,
//     source: I,
//     phantom: std::marker::PhantomData<S>,
// }

// impl<S, V, I> Cumulate<S, V, I> {
//     pub fn new(source: I, initial_state: V) -> Self {
//         Self {
//             state: initial_state,
//             source: ,
//             phantom: std::marker::PhantomData,
//         }
//     }
// }

// impl<S> Indicator<S, f64> for Balance where S: Sequence {}

// impl<S, V> IterIndicator<S, f64> for Balance
// where
//     S: Sequence,
// {
// }
