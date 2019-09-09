use super::*;
use crate::granularity::*;
use crate::indicator::cross::*;
use crate::indicator::sma::*;
use crate::indicator::storage::*;
use crate::indicator::vec::*;
use crate::indicator::*;
use crate::seq::*;
use crate::ticket::*;
use crate::time::*;
use crate::transaction::*;
use chrono::prelude::*;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;

pub struct SimpleStrategyBase {}

type Base = SimpleStrategyBase;
pub struct SimpleSmaCrossStrategy {
    base: Base,
    mid_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    bid_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    ask_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    transaction: Rc<RefCell<VecIndicator<TransactionId, SimpleTransaction>>>,
}

impl SimpleSmaCrossStrategy {
    pub fn new(base: Base, time_offset: Time<S5>, tid_offset: TransactionId) -> Self {
        let mid_close = Storage::new(time_offset).into_sync_ptr();
        let bid_close = Storage::new(time_offset).into_sync_ptr();
        let ask_close = Storage::new(time_offset).into_sync_ptr();

        let sma_short = Sma::new(mid_close.clone(), 25);
        let sma_long = Sma::new(mid_close.clone(), 75);
        let sma_cross = Cross::new(sma_short, sma_long);

        let transaction = VecIndicator::new(tid_offset, vec![]).into_sync_ptr();

        let mut single_ticket = SingleSimpleTicket::new();
        let position_tid = transaction.clone().into_iter(tid_offset).map(move |t| {
            single_ticket.apply_transaction(t);
            single_ticket.as_position()
        });

        // let position_time = 
        // let signal = 

        Self {
            base: base,
            mid_close: mid_close,
            bid_close: bid_close,
            ask_close: ask_close,
            transaction: transaction,
        }
    }

    pub fn update_source(
        &mut self,
        time: DateTime<Utc>,
        mid_close: f64,
        bid_close: f64,
        ask_close: f64,
    ) {
        match <Time<S5>>::try_from(time) {
            Ok(t) => {
                self.mid_close.borrow_mut().add(t, mid_close);
                self.bid_close.borrow_mut().add(t, bid_close);
                self.ask_close.borrow_mut().add(t, ask_close);
            }
            Err(_) => panic!("invalid time"),
        }
    }

    pub fn on_tick(&mut self, time: DateTime<Utc>) {}
}
