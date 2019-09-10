use super::*;
use crate::transaction::*;
use chrono::prelude::*;

// pub struct Trade<T> {
//     pub open: T,
//     pub close: T,
// }

#[derive(Clone, PartialEq, Debug)]
pub struct Trade {
    pub unit: usize,
    pub long_or_short: LongOrShort,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open_price: f64,
    pub close_price: f64,
}

pub struct TradeHistories<I> {
    source: I,
}

impl<I> TradeHistories<I> {
    pub fn new(source: I) -> Self {
        Self { source: source }
    }
}

impl<I> Indicator for TradeHistories<I>
where
    I: Indicator<Seq = TransactionId>,
{
    type Seq = I::Seq;
    type Val = Option<Trade>;
}

impl<I> FuncIndicator for TradeHistories<I>
where
    I: FuncIndicator<Seq = TransactionId, Val = SimpleTransaction>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        match try_value!(self.source.value(seq)) {
            SimpleTransaction::CloseOrderFill(close) => {
                match try_value!(self.source.value(close.open_id)) {
                    SimpleTransaction::OpenOrderFill(open) => MaybeValue::Value(Some(Trade {
                        unit: close.unit,
                        long_or_short: open.ticket.long_or_short,
                        open_time: open.time,
                        close_time: close.time,
                        open_price: open.ticket.price,
                        close_price: close.price,
                    })),
                    _ => panic!("invalid transaction"),
                }
            }
            _ => MaybeValue::Value(None),
        }
    }
}
// impl<S, T, I> Indicator<S, Trade<T>> for TradeHistories<S, I> where S: Sequence {}

// impl<S, T, I> FuncIndicator<S, Trade<T>> for TradeHistories<S, I>
// where
//     S: Sequence,
//     T: Transaction,
//     I: FuncIndicator<S, T>,
// {
//     fn value(&self, seq: S) -> MaybeValue<Trade<T>> {
//         MaybeValue::OutOfRange
//     }
// }

// impl<S, T, I> Indicator<S, Box<[Trade<T>]>> for TradeHistories<S, I> where S: Sequence {}

// impl<S, T, I> FuncIndicator<S, Box<[Trade<T>]>> for TradeHistories<S, I>
// where
//     S: Sequence,
//     T: Transaction,
//     I: FuncIndicator<S, Box<[T]>>,
// {
//     fn value(&self, seq: S) -> MaybeValue<Box<[Trade<T>]>> {
//         MaybeValue::OutOfRange
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::ticket::*;
    use crate::vec::*;
    use LongOrShort::*;

    #[test]
    fn test_trade_tid() {
        let offset = TransactionId(10);
        let time = Time::<S5>::new(0);
        let source = VecIndicator::new(
            offset,
            vec![
                SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                    id: offset + 0,
                    time: (time + 0).into(),
                    ticket: SimpleTicket {
                        id: TicketId(3),
                        open_time: (time + 0).into(),
                        unit: 100,
                        price: 1.234,
                        long_or_short: Long,
                    },
                }),
                SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                    id: offset + 1,
                    time: (time + 3).into(),
                    ticket: SimpleTicket {
                        id: TicketId(4),
                        open_time: (time + 3).into(),
                        unit: 100,
                        price: 1.4,
                        long_or_short: Short,
                    },
                }),
                SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                    id: offset + 2,
                    open_id: offset + 0,
                    time: (time + 5).into(),
                    ticket_id: TicketId(3),
                    unit: 100,
                    price: 1.5,
                }),
                SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                    id: offset + 3,
                    open_id: offset + 1,
                    time: (time + 9).into(),
                    ticket_id: TicketId(4),
                    unit: 100,
                    price: 1.1,
                }),
            ],
        );

        let trade = TradeHistories::new(source);
        assert_eq!(trade.value(offset + 0), MaybeValue::Value(None));
        assert_eq!(trade.value(offset + 1), MaybeValue::Value(None));
        assert_eq!(
            trade.value(offset + 2),
            MaybeValue::Value(Some(Trade {
                unit: 100,
                long_or_short: Long,
                open_time: (time + 0).into(),
                close_time: (time + 5).into(),
                open_price: 1.234,
                close_price: 1.5,
            }))
        );
        assert_eq!(
            trade.value(offset + 3),
            MaybeValue::Value(Some(Trade {
                unit: 100,
                long_or_short: Short,
                open_time: (time + 3).into(),
                close_time: (time + 9).into(),
                open_price: 1.4,
                close_price: 1.1,
            }))
        );
    }
}
