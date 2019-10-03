use super::*;
use chrono::prelude::*;
use log::*;
use MaybeFixed::*;
use MaybeInRange::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::seq::*;
    use crate::vec::*;

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

// #[derive(Clone, Debug)]
// pub struct Candle {
//     open: f64,
//     high: f64,
//     low: f64,
//     close: f64,
// }

// impl Candle {
//     pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
//         Self {
//             open: open,
//             high: high,
//             low: low,
//             close: close,
//         }
//     }
// }

// pub struct IntoTick<IC, IV> {
//     candle: IC,
//     volume: IV,
// }

// impl<IC, IV> IntoTick<IC, IV> {
//     pub fn new(candle: IC, volume: IV) -> Self {
//         Self {
//             candle: candle,
//             volume: volume,
//         }
//     }
// }

// impl<S, IC, IV> Indicator for IntoTick<IC, IV>
// where
//     S: Sequence,
//     IC: Indicator<Seq = S, Val = Option<Candle>>,
//     IV: Indicator<Seq = S, Val = i32>,
// {
//     type Seq = TickId;
//     type Val = (DateTime<Utc>, f64);
// }

// impl<S, IC, IV> FuncIndicator for IntoTick<IC, IV>
// where
//     S: Sequence,
//     IC: FuncIndicator<Seq = S, Val = Option<Candle>>,
//     IV: FuncIndicator<Seq = S, Val = i32>,
// {
//     fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
//         // 多分トップダウンにやったほうがいいので、先に strategy から作る
//         NotFixed
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::granularity::*;
//     use crate::vec::*;

//     #[test]
//     #[ignore]
//     fn test() {
//         let offset = Time::<S5>::new(0);
//         let src_v = vec![1, 0, 2];
//         let src_c = vec![
//             Some(Candle::new(1.0, 1.0, 1.0, 1.0)),
//             None,
//             Some(Candle::new(1.2, 2.0, 1.2, 2.0)),
//         ];
//         let expect = vec![
//             Fixed(InRange((
//                 "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
//                 1.0,
//             ))),
//             Fixed(InRange((
//                 "2019-01-01T00:00:05Z".parse::<DateTime<Utc>>().unwrap(),
//                 1.2,
//             ))),
//             Fixed(InRange((
//                 "2019-01-01T00:00:08Z".parse::<DateTime<Utc>>().unwrap(),
//                 2.0,
//             ))),
//         ];
//         let src_v = VecIndicator::new(offset, src_v);
//         let src_c = VecIndicator::new(offset, src_c);

//         let into_tick = IntoTick::new(src_c, src_v);

//         let result = (0..2)
//             .map(|i| into_tick.value(TickId(i)))
//             .collect::<Vec<_>>();
//         assert_eq!(result, expect);
//     }
// }
