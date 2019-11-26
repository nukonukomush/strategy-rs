use super::busena::signal::*;
use super::busena::status::*;
use super::busena::zone::*;
use super::*;
use crate::granularity::*;
use crate::indicator::balance::*;
use crate::indicator::cached::*;
use crate::indicator::complement::*;
use crate::indicator::convert_seq::*;
use crate::indicator::convert_seq::*;
use crate::indicator::count::*;
use crate::indicator::cross::*;
use crate::indicator::ema::*;
use crate::indicator::envelope::*;
use crate::indicator::sma::*;
use crate::indicator::storage::*;
use crate::indicator::tick::*;
use crate::indicator::trade::*;
use crate::indicator::vec::*;
use crate::indicator::FuncIndicator;
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
    envelopes_p: Vec<Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>>,
    envelopes_m: Vec<Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>>,
    envelopes_tick_p: Vec<Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = f64>>>>,
    envelopes_tick_m: Vec<Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = f64>>>>,

    // zone
    zone_tick: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = ZoneId>>>,

    // up_down
    up_down: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = UpDown>>>,
    up_down_count: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = i32>>>,

    // status
    outermost_zone: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = ZoneId>>>,
    is_entried: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = bool>>>,

    // signal
    signal: Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = LotSignal>>>,
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
        let n_period = 20;
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

        // up_down
        let up_down = up_down(mid_tick.clone()).into_sync_ptr();
        let up_down_count = CountContinuousSameValues::new(up_down.clone(), 20).into_sync_ptr();

        // status
        let outermost_zone = OutermostZone::new(zone_tick.clone(), 20).into_sync_ptr();
        let is_entried = IsEntriedInZone::new(
            outermost_zone.clone(),
            up_down.clone(),
            up_down_count.clone(),
            20,
        )
        .into_sync_ptr();

        // signal
        let signal = Signal::new(outermost_zone.clone(), is_entried.clone()).into_sync_ptr();

        Self {
            latest_time: None,
            time_tick: time_tick,
            mid_tick: mid_tick,
            mid_close_m1: mid_close_m1,
            cmpl_mid_close_m1: cmpl_mid_close_m1,
            sma_mid_close_m1: sma_mid_close_m1,
            ema_mid_close_m1: ema_mid_close_m1,
            envelopes_p: envelopes_p
                .into_iter()
                .map(|i| i as Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>)
                .collect::<Vec<_>>(),
            envelopes_m: envelopes_m
                .into_iter()
                .map(|i| i as Rc<RefCell<dyn FuncIndicator<Seq = Time<M1>, Val = f64>>>)
                .collect::<Vec<_>>(),
            envelopes_tick_p: envelopes_tick_p
                .into_iter()
                .map(|i| i as Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = f64>>>)
                .collect::<Vec<_>>(),
            envelopes_tick_m: envelopes_tick_m
                .into_iter()
                .map(|i| i as Rc<RefCell<dyn FuncIndicator<Seq = TickId, Val = f64>>>)
                .collect::<Vec<_>>(),
            zone_tick: zone_tick,
            up_down: up_down,
            up_down_count: up_down_count,
            outermost_zone: outermost_zone,
            is_entried: is_entried,
            signal: signal,
        }
    }

    pub fn add_price_m1(&mut self, time: Time<M1>, mid_close_m1: f64) {
        self.latest_time = Some(time);
        self.mid_close_m1.borrow_mut().add(time, mid_close_m1);
    }

    pub fn get_signal(&mut self, tick_id: TickId) -> MaybeValue<LotSignal> {
        self.signal.borrow().value(tick_id)
    }

    fn on_tick_inner(&mut self, tick_id: TickId, mid: f64, bid: f64, ask: f64) -> MaybeValue<()> {
        let zone = try_value!(self.zone_tick.borrow().value(tick_id));
        let up_down = try_value!(self.up_down.borrow().value(tick_id));
        let up_down_count = try_value!(self.up_down_count.borrow().value(tick_id));
        let outermost_zone = try_value!(self.outermost_zone.borrow().value(tick_id));
        let signal = try_value!(self.signal.borrow().value(tick_id));
        // self.status.update(zone, up_down, up_down_count as usize);

        let env_p1 = try_value!(self.envelopes_tick_p[0].borrow().value(tick_id));
        let env_m1 = try_value!(self.envelopes_tick_m[0].borrow().value(tick_id));
        println!("env_p1: {}, env_m1: {}", env_p1, env_m1);
        println!("zone: {:?}, {:?}", outermost_zone, zone);
        println!("up_down: {:?}, {:?}", up_down, up_down_count);
        println!("signal: {:?}", signal);
        // println!("{:?}", self.status);

        Fixed(InRange(()))
    }

    pub fn on_tick(&mut self, tick_id: TickId, mid: f64, bid: f64, ask: f64) {
        // add time, prices
        self.time_tick.borrow_mut().add(self.latest_time.unwrap());
        self.mid_tick.borrow_mut().add(mid);

        self.on_tick_inner(tick_id, mid, bid, ask);
    }
}
