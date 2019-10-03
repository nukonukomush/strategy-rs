use crate::granularity::*;
use crate::indicator::*;
use crate::seq::*;
use crate::time::*;
use std::cell::RefCell;
use std::rc::Rc;
use MaybeFixed::*;
use MaybeInRange::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneId(i32);

pub struct Zone<I1, I2> {
    price: I1,
    positive_lines: Vec<I2>,
    negative_lines: Vec<I2>,
}

impl<S, I1, I2> Zone<I1, I2>
where
    S: Sequence,
    I1: FuncIndicator<Seq = S, Val = f64>,
    I2: FuncIndicator<Seq = S, Val = f64>,
{
    pub fn new(price: I1, positive_lines: Vec<I2>, negative_lines: Vec<I2>) -> Self {
        Self {
            price: price,
            positive_lines: positive_lines,
            negative_lines: negative_lines,
        }
    }

    fn check_positive(&self, seq: S) -> MaybeValue<Option<ZoneId>> {
        let price = try_value!(self.price.value(seq));
        for (i, ind) in self.positive_lines.iter().enumerate() {
            let value = try_value!(ind.value(seq));
            if price <= value {
                if i == 0 {
                    return Fixed(InRange(None));
                } else {
                    return Fixed(InRange(Some(ZoneId(i as i32))));
                }
            }
        }
        Fixed(InRange(Some(ZoneId(self.positive_lines.len() as i32))))
    }

    fn check_negative(&self, seq: S) -> MaybeValue<Option<ZoneId>> {
        let price = try_value!(self.price.value(seq));
        for (i, ind) in self.negative_lines.iter().enumerate() {
            let value = try_value!(ind.value(seq));
            if value <= price {
                if i == 0 {
                    return Fixed(InRange(None));
                } else {
                    return Fixed(InRange(Some(ZoneId(-1 * i as i32))));
                }
            }
        }
        Fixed(InRange(Some(ZoneId(-1 * self.negative_lines.len() as i32))))
    }
}

impl<S, I1, I2> Indicator for Zone<I1, I2>
where
    S: Sequence,
    I1: Indicator<Seq = S, Val = f64>,
    I2: Indicator<Seq = S, Val = f64>,
{
    type Seq = S;
    type Val = ZoneId;
}

impl<S, I1, I2> FuncIndicator for Zone<I1, I2>
where
    S: Sequence,
    I1: FuncIndicator<Seq = S, Val = f64>,
    I2: FuncIndicator<Seq = S, Val = f64>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let p_zone = try_value!(self.check_positive(seq));
        let n_zone = try_value!(self.check_negative(seq));
        match (p_zone, n_zone) {
            (Some(_), Some(_)) => panic!(""),
            (Some(z), None) => Fixed(InRange(z)),
            (None, Some(z)) => Fixed(InRange(z)),
            (None, None) => Fixed(InRange(ZoneId(0))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpDown {
    Up,
    Eq,
    Down,
}

use crate::indicator::rolling::*;
pub fn up_down<S, I>(source: I) -> impl FuncIndicator<Seq = S, Val = UpDown>
where
    S: Sequence,
    I: FuncIndicator<Seq = S, Val = f64>,
{
    source.rolling(2, |w| {
        let diff = try_value!(w.rfold(0.0, |x, acc| x - acc));
        let ud = if diff > 0.0 {
            UpDown::Up
        } else if diff < 0.0 {
            UpDown::Down
        } else {
            UpDown::Eq
        };
        Fixed(InRange(ud))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HigeLength(i32);

pub struct HigeSignal<I> {
    price: I,
}

impl<I> HigeSignal<I> {
    pub fn new(price: I) -> Self {
        Self { price: price }
    }
}

impl<I> Indicator for HigeSignal<I>
where
    I: Indicator,
{
    type Seq = I::Seq;
    type Val = HigeLength;
}

impl<I> FuncIndicator for HigeSignal<I>
where
    I: FuncIndicator,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        NotFixed
    }
}

// TODO: ヒゲ判定作成
// TODO: python export
// TODO: python test

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::seq::*;
    use crate::vec::*;

    #[test]
    fn test_sma() {
        let offset = TickId(0);
        let expect = vec![
            Fixed(InRange(ZoneId(0))),
            Fixed(InRange(ZoneId(1))),
            Fixed(InRange(ZoneId(-1))),
            Fixed(InRange(ZoneId(2))),
            Fixed(InRange(ZoneId(-2))),
        ];

        let price = VecIndicator::new(offset, vec![1.0, 2.15, 2.85, 4.3, 4.7]);
        let env_p2 = VecIndicator::new(offset, vec![1.2, 2.2, 3.2, 4.2, 5.2]);
        let env_p1 = VecIndicator::new(offset, vec![1.1, 2.1, 3.1, 4.1, 5.1]);
        let env_m1 = VecIndicator::new(offset, vec![0.9, 1.9, 2.9, 3.9, 4.9]);
        let env_m2 = VecIndicator::new(offset, vec![0.8, 1.8, 2.8, 3.8, 4.8]);

        let zone = Zone::new(price, vec![env_p1, env_p2], vec![env_m1, env_m2]);

        let result = (0..5).map(|i| zone.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_up_down() {
        use UpDown::*;
        let offset = TickId(0);
        let expect = vec![
            Fixed(OutOfRange),
            Fixed(InRange(Up)),
            Fixed(InRange(Down)),
            Fixed(InRange(Down)),
            Fixed(InRange(Down)),
            Fixed(InRange(Up)),
            Fixed(InRange(Up)),
            Fixed(InRange(Up)),
            Fixed(InRange(Eq)),
            Fixed(InRange(Down)),
        ];

        let price = VecIndicator::new(
            offset,
            vec![1.0, 1.1, 1.0, 0.9, 0.8, 1.0, 1.1, 1.2, 1.2, 1.0],
        );

        let ud = up_down(price);

        let result = (0..10).map(|i| ud.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    // // TODO: データ型は要検討
    // #[test]
    // fn test_hige() {
    //     let offset = TickId(0);
    //     let expect = vec![
    //         Fixed(OutOfRange),
    //         Fixed(OutOfRange),
    //         Fixed(InRange(HigeLength(1))),
    //         Fixed(InRange(HigeLength(2))),
    //         Fixed(InRange(HigeLength(3))),
    //         Fixed(InRange(HigeLength(-1))),
    //         Fixed(InRange(HigeLength(-2))),
    //         Fixed(InRange(HigeLength(-3))),
    //         Fixed(InRange(HigeLength(-3))),
    //         Fixed(InRange(HigeLength(1))),
    //     ];

    //     let price = VecIndicator::new(
    //         offset,
    //         vec![1.0, 1.1, 1.0, 0.9, 0.8, 1.0, 1.1, 1.2, 1.2, 1.0],
    //     );

    //     let hige = HigeSignal::new(price);

    //     let result = (0..10).map(|i| hige.value(offset + i)).collect::<Vec<_>>();
    //     assert_eq!(result, expect);
    // }
}
