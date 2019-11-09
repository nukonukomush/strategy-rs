use super::busena::zone::*;
use super::*;
use crate::granularity::*;
use crate::indicator::balance::*;
use crate::indicator::cached::*;
use crate::indicator::complement::*;
use crate::indicator::convert_seq::*;
use crate::indicator::convert_seq::*;
use crate::indicator::cross::*;
use crate::indicator::ema::*;
use crate::indicator::envelope::*;
use crate::indicator::sma::*;
use crate::indicator::storage::*;
use crate::indicator::tick::*;
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
    // time
    time_tick: Rc<RefCell<VecIndicator<TickId, Time<M1>>>>,
    latest_time: Option<Time<M1>>,

    // price
    mid_close_m1: Rc<RefCell<Storage<Time<M1>, f64>>>,
    cmpl_mid_close_m1: Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>,
    mid_tick: Rc<RefCell<VecIndicator<TickId, f64>>>,

    // envelope
    sma_mid_close_m1: Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>,
    ema_mid_close_m1: Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>,
    // envelopes_p: Vec<Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>>,
    // envelopes_m: Vec<Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>>,
    // envelopes_tick_p: Vec<Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = f64>>>>,
    // envelopes_tick_m: Vec<Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = f64>>>>,

    // zone
    zone_tick: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = ZoneId>>>,

    // status
    zone_status: ZoneId,
    trade_status: bool,
    up_down_status: (i32, i32),
}

impl BusenaScalpingStrategy {
    pub fn new(offset_time: Time<M1>) -> Self {
        let offset_tick = TickId(0);

        // TODO: キャッシュ入れる

        // time
        let time_tick = VecIndicator::new(offset_tick, vec![]).into_sync_ptr();

        // price
        let mid_tick = VecIndicator::new(offset_tick, vec![]).into_sync_ptr();
        let mid_close_m1 = Storage::new(offset_time).into_sync_ptr();
        let cmpl_mid_close_m1 =
            ComplementWithLastValue::new(mid_close_m1.clone(), 300).into_sync_ptr();

        // envelope
        let sma_mid_close_m1 = sma(cmpl_mid_close_m1.clone(), 5).into_sync_ptr();
        let n_period = 3;
        let accuracy = 0.9;
        let capacity = 100;
        let ema_mid_close_m1 = LRUCache::new(
            100,
            Ema::new(
                cmpl_mid_close_m1.clone(),
                sma_mid_close_m1.clone(),
                n_period,
                accuracy,
                capacity,
            ),
        )
        .into_sync_ptr();
        let envelopes_p = vec![
            Envelope::new(ema_mid_close_m1.clone(), 0.10).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), 0.15).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), 0.20).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), 0.25).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), 0.30).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), 0.40).into_sync_ptr(),
        ];
        let envelopes_m = vec![
            Envelope::new(ema_mid_close_m1.clone(), -0.10).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), -0.15).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), -0.20).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), -0.25).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), -0.30).into_sync_ptr(),
            Envelope::new(ema_mid_close_m1.clone(), -0.40).into_sync_ptr(),
        ];
        let envelopes_tick_p = envelopes_p
            .iter()
            .map(|i| TimeToId::new(i.clone(), time_tick.clone()).into_sync_ptr())
            .collect::<Vec<_>>();
        let envelopes_tick_m = envelopes_m
            .iter()
            .map(|i| TimeToId::new(i.clone(), time_tick.clone()).into_sync_ptr())
            .collect::<Vec<_>>();

        // zone
        let zone_tick = Zone::new(
            mid_tick.clone(),
            envelopes_tick_p.clone(),
            envelopes_tick_m.clone(),
        )
        .into_sync_ptr();
        // let zone_st_tick = zone_tick.clone().rolling(2, );

        Self {
            latest_time: None,
            time_tick: time_tick,
            mid_tick: mid_tick,
            mid_close_m1: mid_close_m1,
            cmpl_mid_close_m1: cmpl_mid_close_m1,
            sma_mid_close_m1: sma_mid_close_m1,
            ema_mid_close_m1: ema_mid_close_m1,
            // envelopes_p: envelopes_p,
            // envelopes_m: envelopes_m,
            // envelopes_tick_p: envelopes_tick_p,
            // envelopes_tick_m: envelopes_tick_m,
            zone_tick: zone_tick,
            zone_status: ZoneId(0),
            trade_status: false,
            up_down_status: (0, 0),
        }
    }

    pub fn add_price_m1(&mut self, time: Time<M1>, mid_close_m1: f64) {
        self.latest_time = Some(time);
        self.mid_close_m1.borrow_mut().add(time, mid_close_m1);
    }

    pub fn on_tick(&mut self, tick_id: TickId, time: Time<M1>, mid: f64, bid: f64, ask: f64) {
        // add time, prices
        self.time_tick.borrow_mut().add(time);
        self.mid_tick.borrow_mut().add(mid);

        // calc status
        let prev_tick = tick_id - 1;
        let diff = mid - self.mid_tick.value(prev_tick).unwrap().unwrap();
        let mut up_down = 0;
        if diff > 0.0 {
            up_down = 1;
        } else if diff < 0.0 {
            up_down = -1;
        }
        let mut cnt = self.up_down_status.1;
        if self.up_down_status.0 == up_down {
            cnt += 1;
        } else {
            cnt = 1;
        }
        self.up_down_status = (up_down, cnt);
        let zone = self.zone_tick.borrow().value(tick_id).unwrap().unwrap();

        // zone_status は進行するか 0 に戻るかしかない
        // zone_status が変わると trade_status がリセットされる
        let mut entry_signal = 0;
        if self.zone_status.0 > 0 {
            if zone.0 > self.zone_status.0 || zone.0 == 0 {
                self.zone_status = zone;
                self.trade_status = false;
            } else if self.trade_status == false {
                if self.zone_status.0 == 1 || self.zone_status.0 == 2 {
                    if self.up_down_status == (-1, 1) {
                        // sell
                        entry_signal = self.zone_status.0 * -1;
                        self.trade_status = true;
                    }
                } else if self.zone_status.0 == 3
                    || self.zone_status.0 == 4
                    || self.zone_status.0 == 5
                {
                    if self.up_down_status == (-1, 2) {
                        // sell
                        entry_signal = self.zone_status.0 * -1;
                        self.trade_status = true;
                    }
                }
            }
        } else if self.zone_status.0 < 0 {
            if zone.0 < self.zone_status.0 || zone.0 == 0 {
                self.zone_status = zone;
                self.trade_status = false;
            } else if self.trade_status == false {
                if self.zone_status.0 == -1 || self.zone_status.0 == -2 {
                    if self.up_down_status == (1, 1) {
                        // buy
                        entry_signal = self.zone_status.0 * -1;
                        self.trade_status = true;
                    }
                } else if self.zone_status.0 == -3
                    || self.zone_status.0 == -4
                    || self.zone_status.0 == -5
                {
                    if self.up_down_status == (1, 2) {
                        // buy
                        entry_signal = self.zone_status.0 * -1;
                        self.trade_status = true;
                    }
                }
            }
        } else {
            self.zone_status = zone;
        }

        // order
        if entry_signal != 0 {
            let units = entry_signal * 100_000;
            let pip = mid / 10000.0;
            if entry_signal > 0 {
                let price = ask;
                let stop_loss = bid - pip * (entry_signal.abs() + 1) as f64;
                let take_profit = price + pip * (entry_signal.abs() + 1) as f64;
            // order(units, price, stop_loss, take_profit);
            } else {
                let price = bid;
                let stop_loss = ask + pip * (entry_signal.abs() + 1) as f64;
                let take_profit = price - pip * (entry_signal.abs() + 1) as f64;
                // order(units, price, stop_loss, take_profit);
            }
        }
    }
}

// #[derive(PartialEq, Eq, Debug)]
// pub enum UpDown {
//     Neutral,
//     Up,
//     Down,
// }


pub fn zone_st<I>(zone: I) -> impl FuncIndicator
where
    I: FuncIndicator<Val = ZoneId>,
{
    // zone.rolling(2, |w| {
    //     let z0 = try_value!(w.value(0));
    //     let z1 = try_value!(w.value(1));
    // })
    ClosureIndicator::new(move |seq| {
        let z1 = try_value!(zone.value(seq - 1));
        let z2 = try_value!(zone.value(seq));

        if z1.0 == 0 {
            Fixed(InRange(z2))
        } else if z2.0 > z1.0 && z1.0 > 0 {
            Fixed(InRange(z2))
        } else if z2.0 < z1.0 && z1.0 < 0 {
            Fixed(InRange(z2))
        } else {
            Fixed(InRange(z1))
        }
    })
}

