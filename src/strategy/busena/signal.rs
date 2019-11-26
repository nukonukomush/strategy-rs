use super::status::*;
use super::zone::*;
use crate::indicator::MaybeFixed::*;
use crate::indicator::MaybeInRange::*;
use crate::indicator::*;
use crate::seq::*;

#[derive(Debug, PartialEq, Eq)]
pub enum LotSignal {
    Nothing,
    Buy(usize),
    Sell(usize),
}

pub struct Signal<I1, I2> {
    outermost_zone: I1,
    is_entried: I2,
}

impl<S, I1, I2> Signal<I1, I2>
where
    S: Sequence,
    I1: Indicator<Seq = S, Val = ZoneId>,
    I2: Indicator<Seq = S, Val = bool>,
{
    pub fn new(outermost_zone: I1, is_entried: I2) -> Self {
        Self {
            outermost_zone: outermost_zone,
            is_entried: is_entried,
        }
    }
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

impl<S, I1, I2> Indicator for Signal<I1, I2>
where
    S: Sequence,
    I1: Indicator<Seq = S, Val = ZoneId>,
    I2: Indicator<Seq = S, Val = bool>,
{
    type Seq = S;
    type Val = LotSignal;
}

impl<S, I1, I2> FuncIndicator for Signal<I1, I2>
where
    S: Sequence,
    I1: FuncIndicator<Seq = S, Val = ZoneId>,
    I2: FuncIndicator<Seq = S, Val = bool>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let prev_is_entried = try_value!(self.is_entried.value(seq - 1));
        let curr_is_entried = try_value!(self.is_entried.value(seq));
        let outermost_zone = try_value!(self.outermost_zone.value(seq));

        // false => true ならエントリー
        let signal = if prev_is_entried == false && curr_is_entried == true {
            let zone = outermost_zone;
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

#[cfg(test)]
mod tests {
    use super::super::status::tests::*;
    use super::*;
    use crate::granularity::*;
    use crate::time::*;
    use crate::vec::*;

    #[test]
    fn test_signal_1() {
        let offset = Time::<S5>::new(0);
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Sell(10))),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Nothing)),
        ];
        let is_entried = VecIndicator::new(offset, vec![false, false, true, true, false]);
        let zone = VecIndicator::new(offset, vec![ZoneId(1); 5]);
        let signal = Signal::new(zone, is_entried);

        let result = (0..5).map(|i| signal.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_signal_2() {
        let offset = Time::<S5>::new(0);
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Buy(30))),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Nothing)),
        ];
        let is_entried = VecIndicator::new(offset, vec![false, false, true, true, false]);
        let zone = VecIndicator::new(offset, vec![ZoneId(-3); 5]);
        let signal = Signal::new(zone, is_entried);

        let result = (0..5).map(|i| signal.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    // TODO: この仕様は微妙なので直したい
    #[test]
    fn test_signal_3() {
        let offset = Time::<S5>::new(0);
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Nothing)),
            Fixed(InRange(LotSignal::Nothing)),
        ];
        let is_entried = VecIndicator::new(offset, vec![false, false, true, true, false]);
        let zone = VecIndicator::new(offset, vec![ZoneId(6); 5]);
        let signal = Signal::new(zone, is_entried);

        let result = (0..5).map(|i| signal.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
