use super::status::*;
use super::zone::*;
use crate::indicator::MaybeFixed::*;
use crate::indicator::MaybeInRange::*;
use crate::indicator::*;
use crate::seq::*;

#[derive(Debug)]
pub enum LotSignal {
    Nothing,
    Buy(usize),
    Sell(usize),
}

pub struct Signal<I> {
    state: I,
}

pub fn lot_by_zone(zone: ZoneId) -> usize {
    let abs_zone = zone.0.abs();
    match abs_zone {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        _ => 0,
    }
}

pub fn pips_range_by_zone(zone: ZoneId) -> usize {
    let abs_zone = zone.0.abs();
    match abs_zone {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 5,
        5 => 6,
        _ => 0,
    }
}

impl<I> Signal<I> {}

impl<S, I> Indicator for Signal<I>
where
    S: Sequence,
    I: Indicator<Seq = S, Val = Status>,
{
    type Seq = S;
    type Val = LotSignal;
}

impl<S, I> FuncIndicator for Signal<I>
where
    S: Sequence,
    I: FuncIndicator<Seq = S, Val = Status>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let prev_state = try_value!(self.state.value(seq - 1));
        let curr_state = try_value!(self.state.value(seq));

        // false => true ならエントリー
        let signal = if prev_state.is_entried() == false && curr_state.is_entried() == true {
            let zone = curr_state.outermost_zone();
            let lot = lot_by_zone(zone);
            if lot != 0 {
                if zone.0 > 0 {
                    LotSignal::Sell(lot)
                } else {
                    LotSignal::Buy(lot)
                }
            } else {
                LotSignal::Nothing
            }
        } else {
            LotSignal::Nothing
        };
        Fixed(InRange(signal))
    }
}

// pub fn signal<I>(state: I) -> impl FuncIndicator<Val = LotSignal>
// where
//     I: FuncIndicator<Val = Status>,
// {
//     state.rolling(2, )
// }
