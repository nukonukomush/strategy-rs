use crate::position::*;
use crate::seq::*;
use crate::transaction::*;
use chrono::prelude::*;
use std::ops::Add;
use std::ops::Sub;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct TicketId(pub i64);

impl Add<i64> for TicketId {
    type Output = TicketId;
    fn add(self, other: i64) -> Self::Output {
        TicketId(self.0 + other)
    }
}

impl Sub<i64> for TicketId {
    type Output = TicketId;
    fn sub(self, other: i64) -> Self::Output {
        TicketId(self.0 - other)
    }
}

impl Sequence for TicketId {
    fn distance_from(&self, offset: &Self) -> i64 {
        self.0 - offset.0
    }
}

impl Into<i64> for TicketId {
    fn into(self) -> i64 {
        self.0
    }
}

impl From<i64> for TicketId {
    fn from(i: i64) -> Self {
        TicketId(i)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SimpleTicket {
    pub id: TicketId,
    pub open_time: DateTime<Utc>,
    pub unit: usize,
    pub price: f64,
    pub long_or_short: LongOrShort,
}

pub struct SingleSimpleTicket {
    ticket: Option<SimpleTicket>,
}

impl SingleSimpleTicket {
    pub fn new() -> Self {
        Self { ticket: None }
    }

    // TODO: unuse panic
    pub fn apply_transaction(&mut self, transaction: SimpleTransaction) {
        match transaction {
            SimpleTransaction::OpenOrderFill(t) => match self.ticket {
                Some(_) => panic!("invalid transaction"),
                None => self.ticket = Some(t.ticket),
            },
            SimpleTransaction::CloseOrderFill(t) => match &mut self.ticket {
                Some(ticket) => {
                    if t.ticket_id == ticket.id {
                        self.ticket = None;
                    } else {
                        panic!("invalid ticket_id: {:?}", t.ticket_id)
                    }
                }
                None => panic!("invalid transaction"),
            },
            _ => (),
        }
    }

    pub fn ticket(&self) -> Option<SimpleTicket> {
        self.ticket.clone()
    }

    pub fn as_position(&self) -> SimplePosition {
        match &self.ticket {
            Some(ticket) => match ticket.long_or_short {
                LongOrShort::Long => SimplePosition::Long,
                LongOrShort::Short => SimplePosition::Short,
            },
            None => SimplePosition::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::indicator::*;
    use crate::seq::*;
    use crate::time::*;
    use crate::vec::*;
    use LongOrShort::*;

    fn get_source(
        offset: TransactionId,
        time: Time<S5>,
    ) -> impl FuncIndicator<Seq = TransactionId, Val = SimpleTransaction> {
        VecIndicator::new(
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
                SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                    id: offset + 1,
                    open_id: offset + 0,
                    time: (time + 5).into(),
                    ticket_id: TicketId(3),
                    unit: 100,
                    price: 1.5,
                }),
                SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                    id: offset + 2,
                    time: (time + 7).into(),
                    ticket: SimpleTicket {
                        id: TicketId(4),
                        open_time: (time + 7).into(),
                        unit: 100,
                        price: 1.4,
                        long_or_short: Short,
                    },
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
        )
    }

    #[test]
    fn test_single_ticket() {
        let offset = TransactionId(10);
        let time = Time::<S5>::new(0);

        let source = get_source(offset, time);
        let mut single_ticket = SingleSimpleTicket::new();
        let tickets = source
            .into_iter(offset)
            .map(move |t| {
                single_ticket.apply_transaction(t);
                single_ticket.ticket()
            })
            .into_std()
            .collect::<Vec<_>>();
        let expect = vec![
            Some(SimpleTicket {
                id: TicketId(3),
                open_time: (time + 0).into(),
                unit: 100,
                price: 1.234,
                long_or_short: Long,
            }),
            None,
            Some(SimpleTicket {
                id: TicketId(4),
                open_time: (time + 7).into(),
                unit: 100,
                price: 1.4,
                long_or_short: Short,
            }),
            None,
        ];
        assert_eq!(tickets, expect);
    }

    #[test]
    fn test_position() {
        let offset = TransactionId(10);
        let time = Time::<S5>::new(0);

        let source = get_source(offset, time);
        let mut single_ticket = SingleSimpleTicket::new();
        let positions = source
            .into_iter(offset)
            .map(move |t| {
                single_ticket.apply_transaction(t);
                single_ticket.as_position()
            })
            .into_std()
            .collect::<Vec<_>>();
        let expect = vec![
            SimplePosition::Long,
            SimplePosition::Nothing,
            SimplePosition::Short,
            SimplePosition::Nothing,
        ];
        assert_eq!(positions, expect);
    }
}
