use crate::granularity::*;
use crate::indicator::*;
use crate::seq::*;
use crate::time::*;
use std::cell::RefCell;
use std::rc::Rc;
use MaybeFixed::*;
use MaybeInRange::*;

// type Ind<S> = Rc<RefCell<dyn FuncIndicator<Seq = S, Val = f64>>>;

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

pub struct TimeToId<IV, IT> {
    values: IV,
    time: IT,
}

impl<IV, IT> TimeToId<IV, IT> {
    pub fn new(values: IV, time: IT) -> Self {
        Self {
            values: values,
            time: time,
        }
    }
}

impl<T, IV, IT> Indicator for TimeToId<IV, IT>
where
    T: Sequence,
    IV: Indicator<Seq = T>,
    IT: Indicator<Val = T>,
{
    type Seq = IT::Seq;
    type Val = IV::Val;
}

impl<T, IV, IT> FuncIndicator for TimeToId<IV, IT>
where
    T: Sequence,
    IV: FuncIndicator<Seq = T>,
    IT: FuncIndicator<Val = T>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let time = try_value!(self.time.value(seq));
        self.values.value(time)
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
    fn test_tick() {
        let offset = Time::<S5>::new(0);
        let expect = vec![
            Fixed(InRange(1.0)),
            Fixed(InRange(2.0)),
            Fixed(InRange(2.0)),
            Fixed(InRange(4.0)),
            Fixed(InRange(5.0)),
            Fixed(InRange(5.0)),
        ];

        let source = VecIndicator::new(offset, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let time = VecIndicator::new(
            TickId(0),
            vec![
                Time::<S5>::new(0),
                Time::<S5>::new(5),
                Time::<S5>::new(5),
                Time::<S5>::new(15),
                Time::<S5>::new(20),
                Time::<S5>::new(20),
            ],
        );

        let time_to_tick = TimeToId::new(source, time);

        let result = (0..6)
            .map(|i| time_to_tick.value(TickId(i)))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
