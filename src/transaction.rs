use chrono::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OpenOrClose {
    Open,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LongOrShort {
    Long,
    Short,
}

#[derive(Clone)]
pub struct SimpleOrderFillTransaction {
    pub id: usize,
    pub time: DateTime<Utc>,
    pub unit: usize,
    pub price: f64,
    pub open_or_close: OpenOrClose,
    pub long_or_short: LongOrShort,
    pub orders_on_fill: Vec<SimpleOrder>,
}

impl Transaction for SimpleOrderFillTransaction {
    fn id(&self) -> usize {
        self.id
    }

    fn time(&self) -> DateTime<Utc> {
        self.time
    }
}

// #[derive(Clone)]
// pub enum SimpleTransaction {
//     OpenOrderFill(OpenOrderFill),
//     CloseOrderFill(CloseOrderFill),
// }

// impl Transaction for SimpleTransaction {
//     fn id(&self) -> usize {
//         match self {
//             SimpleTransaction::OpenOrderFill { id, time, price } => *id,
//             SimpleTransaction::CloseOrderFill { id, time, price } => *id,
//         }
//     }

//     fn time(&self) -> DateTime<Utc> {
//         match self {
//             SimpleTransaction::OpenOrderFill { id, time, price } => *time,
//             SimpleTransaction::CloseOrderFill { id, time, price } => *time,
//         }
//     }
//     fn price(&self) -> f64 {
//         match self {
//             SimpleTransaction::OpenOrderFill { id, time, price } => *price,
//             SimpleTransaction::CloseOrderFill { id, time, price } => *price,
//         }
//     }
// }

// pub trait Transaction {}

pub trait Transaction {
    fn id(&self) -> usize;
    fn time(&self) -> DateTime<Utc>;
    // fn price(&self) -> f64;
}

// pub trait OrderFill: Transaction {
//     fn price(&self) -> f64;
// }

pub struct TransactionHistories<T> {
    histories: Vec<T>,
}

impl<T> TransactionHistories<T>
where
    T: Transaction + Clone,
{
    pub fn new() -> Self {
        Self {
            histories: Vec::new(),
        }
    }

    pub fn push(&mut self, transaction: T) {
        // TODO: debug_assert id, time
        self.histories.push(transaction)
    }

    pub fn latest_time(&self) -> Option<DateTime<Utc>> {
        if self.histories.is_empty() {
            return None;
        }

        let len = self.histories.len();
        Some(self.histories[len - 1].time())
    }

    pub fn get_by_id(&self, id: usize) -> Option<T> {
        if self.histories.is_empty() {
            return None;
        }

        let len = self.histories.len();
        // let latest_id = self.histories[len - 1].id();
        let first_id = self.histories[0].id();
        if id < len + first_id {
            Some(self.histories[id - first_id].clone())
        } else {
            None
        }
    }

    pub fn get_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Option<Box<[T]>> {
        let start_index = match self
            .histories
            .binary_search_by_key(&start, Transaction::time)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        let end_index = match self.histories.binary_search_by_key(&end, Transaction::time) {
            Ok(i) => i,
            Err(i) => i - 1, // TODO: debug overflow
        };

        if start_index < end_index {
            Some(
                self.histories[start_index..end_index]
                    .iter()
                    .map(Clone::clone)
                    .collect::<Box<[T]>>(),
            )
        } else {
            None
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &T> {
        self.histories.iter()
    }
}

#[derive(Clone)]
pub struct SimpleTicket {
    pub transaction_id: usize,
    pub unit: usize,
    pub price: f64,
    pub time: DateTime<Utc>,
    pub long_or_short: LongOrShort,
    pub related_orders: Vec<SimpleOrder>,
}

#[derive(Clone)]
pub struct SimpleTakeProfitOrder {
    pub price: f64,
}
#[derive(Clone)]
pub struct SimpleStopLossOrder {
    pub price: f64,
}
#[derive(Clone)]
pub struct SimpleTrailingStopLossOrder {
    pub distance: f64,
}

#[derive(Clone)]
pub enum SimpleOrder {
    TakeProfit(SimpleTakeProfitOrder),
    StopLoss(SimpleStopLossOrder),
    TrailingStopLoss(SimpleTrailingStopLossOrder),
}

// // TODO: refactor
// pub struct Trade {
//     pub transaction_id: usize,
//     pub unit: usize,
//     pub price: f64,
//     pub time: DateTime<Utc>,
// }

// pub struct TradeHistory {
//     pub long_or_short: LongOrShort,
//     pub open: Trade,
//     pub close: Trade,
// }

// impl TradeHistory {
//     pub fn new(long_or_short: LongOrShort, open: Trade, close: Trade) -> Self {
//         Self {
//             long_or_short: long_or_short,
//             open: open,
//             close: close,
//         }
//     }
// }

// pub struct SingleSimpleTicket {
//     ticket: Option<SimpleTicket>,
// }

// impl SingleSimpleTicket {
//     pub fn new() -> Self {
//         Self { ticket: None }
//     }

//     pub fn apply_transaction(
//         &mut self,
//         transaction: SimpleOrderFillTransaction,
//     ) -> Result<Option<TradeHistory>, ()> {
//         let open_or_close = transaction.open_or_close;
//         let long_or_short = transaction.long_or_short;
//         use LongOrShort::*;
//         use OpenOrClose::*;
//         match (&mut self.ticket, open_or_close, long_or_short) {
//             (Some(ticket), Close, Long) if ticket.long_or_short == Long => {
//                 let ret = Ok(Some(TradeHistory::new(
//                     Long,
//                     Trade {
//                         transaction_id: ticket.transaction_id,
//                         unit: ticket.unit,
//                         price: ticket.price,
//                         time: ticket.time,
//                     },
//                     Trade {
//                         transaction_id: transaction.id,
//                         unit: transaction.unit,
//                         price: transaction.price,
//                         time: transaction.time,
//                     },
//                 )));
//                 self.ticket = None;
//                 ret
//             }
//             (Some(ticket), Close, Short) if ticket.long_or_short == Short => {
//                 let ret = Ok(Some(TradeHistory::new(
//                     Short,
//                     Trade {
//                         transaction_id: ticket.transaction_id,
//                         unit: ticket.unit,
//                         price: ticket.price,
//                         time: ticket.time,
//                     },
//                     Trade {
//                         transaction_id: transaction.id,
//                         unit: transaction.unit,
//                         price: transaction.price,
//                         time: transaction.time,
//                     },
//                 )));
//                 self.ticket = None;
//                 ret
//             }
//             (None, Open, Long) => {
//                 self.ticket = Some(SimpleTicket {
//                     transaction_id: transaction.id,
//                     unit: transaction.unit,
//                     price: transaction.price,
//                     time: transaction.time,
//                     long_or_short: Long,
//                     related_orders: transaction.orders_on_fill,
//                 });
//                 Ok(None)
//             }
//             (None, Open, Short) => {
//                 self.ticket = Some(SimpleTicket {
//                     transaction_id: transaction.id,
//                     unit: transaction.unit,
//                     price: transaction.price,
//                     time: transaction.time,
//                     long_or_short: Short,
//                     related_orders: transaction.orders_on_fill,
//                 });
//                 Ok(None)
//             }
//             _ => Err(()),
//         }
//     }
// }

// pub struct SimpleOrder {
//     pub is_open: bool,
//     pub is_buy: bool,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::cell::RefCell;
//     use std::rc::Rc;

//     #[test]
//     fn test_sum() {
//         let mut histories = TransactionHistories::new();
//         histories.push(SimpleTransaction::Open {
//             id: 1,
//             time: Utc.timestamp(0, 0),
//             price: 1.0,
//         });
//         histories.push(SimpleTransaction::Close {
//             id: 2,
//             time: Utc.timestamp(5, 0),
//             price: 3.0,
//         });
//         histories.push(SimpleTransaction::Open {
//             id: 3,
//             time: Utc.timestamp(10, 0),
//             price: 2.0,
//         });
//         histories.push(SimpleTransaction::Close {
//             id: 4,
//             time: Utc.timestamp(15, 0),
//             price: 3.0,
//         });
//         let sum = Rc::new(RefCell::new(0.0));
//         let sum_move = sum.clone();
//         histories
//             .iter()
//             .for_each(move |t| *sum_move.borrow_mut() += t.price());
//         assert_eq!(*sum.borrow(), );
//     }
// }
