use super::*;
use crate::granularity::*;
use crate::indicator::balance::*;
use crate::indicator::complement::*;
use crate::indicator::convert_seq::*;
use crate::indicator::convert_seq::*;
use crate::indicator::cross::*;
use crate::indicator::sma::*;
use crate::indicator::storage::*;
use crate::indicator::trade::*;
use crate::indicator::vec::*;
use crate::indicator::*;
use crate::position::*;
use crate::seq::*;
use crate::signal::*;
use crate::ticket::*;
use crate::time::*;
use crate::trade::*;
use crate::transaction::*;
use chrono::prelude::*;
use log::*;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct BusenaScalpingStrategy {
    mid_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    bid_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    ask_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    bid_close_cmpl: Rc<RefCell<dyn FuncIndicator<Seq = Time<S5>, Val = f64>>>,
    ask_close_cmpl: Rc<RefCell<dyn FuncIndicator<Seq = Time<S5>, Val = f64>>>,
    transaction: Rc<RefCell<VecIndicator<TransactionId, SimpleTransaction>>>,
    trade: Rc<RefCell<dyn FuncIndicator<Seq = TransactionId, Val = Option<Trade>>>>,
    signal: Box<dyn IterIndicator<Seq = Time<S5>, Val = SimpleSignal>>,
    tid_offset: TransactionId,
    ticket_id_offset: TicketId,
    single_ticket: Rc<RefCell<SingleSimpleTicket>>,
    balance: Box<dyn IterIndicator<Seq = Time<S5>, Val = f64>>,
}
