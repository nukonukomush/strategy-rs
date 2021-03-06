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

pub struct SimpleStrategyBase {}

type Base = SimpleStrategyBase;
pub struct SimpleSmaCrossStrategy {
    base: Base,
    mid_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    bid_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    ask_close: Rc<RefCell<Storage<Time<S5>, f64>>>,
    // mid_close: Rc<RefCell<VecIndicator<Time<S5>, Option<f64>>>>,
    // bid_close: Rc<RefCell<VecIndicator<Time<S5>, Option<f64>>>>,
    // ask_close: Rc<RefCell<VecIndicator<Time<S5>, Option<f64>>>>,
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

impl SimpleSmaCrossStrategy {
    pub fn new(base: Base, time_offset: Time<S5>, tid_offset: TransactionId) -> Self {
        let mid_close = Storage::new(time_offset).into_sync_ptr();
        let bid_close = Storage::new(time_offset).into_sync_ptr();
        let ask_close = Storage::new(time_offset).into_sync_ptr();
        // let mid_close = VecIndicator::new(time_offset, Vec::with_capacity(1000)).into_sync_ptr();
        // let bid_close = VecIndicator::new(time_offset, Vec::with_capacity(1000)).into_sync_ptr();
        // let ask_close = VecIndicator::new(time_offset, Vec::with_capacity(1000)).into_sync_ptr();

        let mid_close_cmpl = ComplementWithLastValue::new(mid_close.clone(), 100).into_sync_ptr();
        let bid_close_cmpl = ComplementWithLastValue::new(bid_close.clone(), 100).into_sync_ptr();
        let ask_close_cmpl = ComplementWithLastValue::new(ask_close.clone(), 100).into_sync_ptr();
        // let mid_close_cmpl_2 = mid_close
        //     .clone()
        //     .map(|v| v.or(Some(0.0)).unwrap())
        //     .when_not_fixed(|| Fixed(InRange(0.0)))
        //     .into_sync_ptr();

        let sma_short = sma(mid_close_cmpl.clone(), 25);
        let sma_long = sma(mid_close_cmpl.clone(), 75);
        let sma_cross = Cross::new(sma_short, sma_long).into_sync_ptr();

        let transaction = VecIndicator::new(tid_offset, vec![]).into_sync_ptr();

        let single_ticket = Rc::new(RefCell::new(SingleSimpleTicket::new()));
        let st = single_ticket.clone();
        let position_tid = transaction.clone().into_iter(tid_offset).map(move |t| {
            st.borrow_mut().apply_transaction(t);
            st.borrow().as_position()
        });
        let position_time = {
            let mut pos = SimplePosition::Nothing;
            Consume::new(time_offset, position_tid, move |i| {
                let v = i.into_std().fold(None, |_, v| Some(v));
                if v.is_some() {
                    pos = v.unwrap();
                }
                // println!("position: {:?}", pos);
                Fixed(InRange(pos))
            })
        };

        let trade = TradeHistories::new(transaction.clone()).into_sync_ptr();
        let pl = ProfitLoss::new(trade.clone()).into_sync_ptr();
        let mut sum = 0.0;
        let balance = Consume::new(time_offset, pl.clone().into_iter(tid_offset), |i| {
            let v = i.into_std().fold(0.0, |acc, v| acc + v);
            Fixed(InRange(v))
        })
        .map(move |v| {
            sum += v;
            // debug!("balance: {:?}", sum);
            if v != 0.0 {
                info!("balance: {:?}", sum);
            }
            sum
        })
        .into_sync_ptr();

        let signal = sma_cross
            .clone()
            .into_iter(time_offset)
            .zip(position_time)
            .map(|(cross, pos)| {
                use CrossState::*;
                use SimplePosition::*;
                match (cross, pos) {
                    (LtToGt, Nothing) => SimpleSignal::OpenLong,
                    (GtToLt, Nothing) => SimpleSignal::OpenShort,
                    (LtToGt, Short) => SimpleSignal::CloseShortAndOpenLong,
                    (GtToLt, Long) => SimpleSignal::CloseLongAndOpenShort,
                    _ => SimpleSignal::Nothing,
                }
            })
            .into_sync_ptr();

        Self {
            base: base,
            mid_close: mid_close,
            bid_close: bid_close,
            ask_close: ask_close,
            bid_close_cmpl: bid_close_cmpl,
            ask_close_cmpl: ask_close_cmpl,
            transaction: transaction,
            trade: trade,
            signal: signal,
            tid_offset: tid_offset,
            ticket_id_offset: TicketId(0),
            single_ticket: single_ticket,
            balance: balance,
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
                // println!("{:?},{:?},{:?}", mid_close, bid_close, ask_close);
                self.mid_close.borrow_mut().add(t, mid_close);
                self.bid_close.borrow_mut().add(t, bid_close);
                self.ask_close.borrow_mut().add(t, ask_close);
            }
            Err(_) => panic!("invalid time"),
        }
    }

    // pub fn update_source(
    //     &mut self,
    //     time: DateTime<Utc>,
    //     mid_close: Option<f64>,
    //     bid_close: Option<f64>,
    //     ask_close: Option<f64>,
    // ) {
    //     match <Time<S5>>::try_from(time) {
    //         Ok(t) => {
    //             // println!("{:?},{:?},{:?}", mid_close, bid_close, ask_close);
    //             self.mid_close.borrow_mut().add(mid_close);
    //             self.bid_close.borrow_mut().add(bid_close);
    //             self.ask_close.borrow_mut().add(ask_close);
    //         }
    //         Err(_) => panic!("invalid time"),
    //     }
    // }

    fn next_tid(&mut self) -> TransactionId {
        let tid = self.tid_offset;
        self.tid_offset = tid + 1;
        tid
    }

    fn next_ticket_id(&mut self) -> TicketId {
        let ticket_id = self.ticket_id_offset;
        self.ticket_id_offset = ticket_id + 1;
        ticket_id
    }

    pub fn on_tick(&mut self, time: DateTime<Utc>) {
        let time_s5 = match <Time<S5>>::try_from(time) {
            Ok(t) => t,
            Err(_) => return,
        };

        self.balance.next();
        let dt: DateTime<Utc> = self.signal.offset().into();
        // println!("signal offset: {:?}", dt);
        let signal = self.signal.next();
        match signal {
            Fixed(InRange(s)) => match s {
                SimpleSignal::Nothing => (),
                other => info!("signal: {:?}", s),
            },
            _ => (),
        }
        use LongOrShort::*;
        match signal {
            Fixed(InRange(s)) => match s {
                SimpleSignal::OpenLong => {
                    let t = SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                        id: self.next_tid(),
                        time: time,
                        ticket: SimpleTicket {
                            id: self.next_ticket_id(),
                            open_time: time,
                            unit: 100,
                            price: self
                                .ask_close_cmpl
                                .borrow()
                                .value(time_s5)
                                .unwrap()
                                .unwrap(),
                            long_or_short: Long,
                        },
                    });
                    self.transaction.borrow_mut().add(t);
                }
                SimpleSignal::OpenShort => {
                    let t = SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                        id: self.next_tid(),
                        time: time,
                        ticket: SimpleTicket {
                            id: self.next_ticket_id(),
                            open_time: time,
                            unit: 100,
                            price: self
                                .bid_close_cmpl
                                .borrow()
                                .value(time_s5)
                                .unwrap()
                                .unwrap(),
                            long_or_short: Short,
                        },
                    });
                    self.transaction.borrow_mut().add(t);
                }
                SimpleSignal::CloseLong => {
                    let tid = self.next_tid();
                    let t = SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                        id: tid,
                        open_id: tid - 1,
                        time: time,
                        ticket_id: self.single_ticket.borrow().ticket().unwrap().id,
                        unit: 100,
                        price: self
                            .bid_close_cmpl
                            .borrow()
                            .value(time_s5)
                            .unwrap()
                            .unwrap(),
                    });
                    self.transaction.borrow_mut().add(t);
                }
                SimpleSignal::CloseShort => {
                    let tid = self.next_tid();
                    let t = SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                        id: tid,
                        open_id: tid - 1,
                        time: time,
                        ticket_id: self.single_ticket.borrow().ticket().unwrap().id,
                        unit: 100,
                        price: self
                            .ask_close_cmpl
                            .borrow()
                            .value(time_s5)
                            .unwrap()
                            .unwrap(),
                    });
                    self.transaction.borrow_mut().add(t);
                }
                SimpleSignal::CloseLongAndOpenShort => {
                    let tid = self.next_tid();
                    let t = SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                        id: tid,
                        open_id: tid - 1,
                        time: time,
                        ticket_id: self.single_ticket.borrow().ticket().unwrap().id,
                        unit: 100,
                        price: self
                            .bid_close_cmpl
                            .borrow()
                            .value(time_s5)
                            .unwrap()
                            .unwrap(),
                    });
                    self.transaction.borrow_mut().add(t);
                    let t = SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                        id: self.next_tid(),
                        time: time,
                        ticket: SimpleTicket {
                            id: self.next_ticket_id(),
                            open_time: time,
                            unit: 100,
                            price: self
                                .bid_close_cmpl
                                .borrow()
                                .value(time_s5)
                                .unwrap()
                                .unwrap(),
                            long_or_short: Short,
                        },
                    });
                    self.transaction.borrow_mut().add(t);
                }
                SimpleSignal::CloseShortAndOpenLong => {
                    let tid = self.next_tid();
                    let t = SimpleTransaction::CloseOrderFill(CloseOrderFillTransaction {
                        id: tid,
                        open_id: tid - 1,
                        time: time,
                        ticket_id: self.single_ticket.borrow().ticket().unwrap().id,
                        unit: 100,
                        price: self
                            .ask_close_cmpl
                            .borrow()
                            .value(time_s5)
                            .unwrap()
                            .unwrap(),
                    });
                    self.transaction.borrow_mut().add(t);
                    let t = SimpleTransaction::OpenOrderFill(OpenOrderFillTransaction {
                        id: self.next_tid(),
                        time: time,
                        ticket: SimpleTicket {
                            id: self.next_ticket_id(),
                            open_time: time,
                            unit: 100,
                            price: self
                                .ask_close_cmpl
                                .borrow()
                                .value(time_s5)
                                .unwrap()
                                .unwrap(),
                            long_or_short: Long,
                        },
                    });
                    self.transaction.borrow_mut().add(t);
                }
                _ => (),
            },
            NotFixed => {
                debug!("signal is not fixed");
            }
            Fixed(OutOfRange) => {
                debug!("signal is out of range");
            }
        };
    }
}
