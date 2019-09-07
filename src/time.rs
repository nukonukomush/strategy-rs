use crate::granularity::*;
use crate::seq::*;
use chrono::prelude::*;
use std::ops::Add;
use std::ops::Sub;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct Time<G>(i64, std::marker::PhantomData<G>);

impl<G> Time<G>
where
    G: StaticGranularity,
{
    pub fn new(t: i64) -> Self {
        debug_assert!(G::is_valid(t));
        Time(t, std::marker::PhantomData)
    }

    pub fn timestamp(&self) -> i64 {
        self.0
    }

    pub fn try_into<G2: StaticGranularity>(self) -> Result<Time<G2>, ()> {
        if G2::is_valid(self.0) {
            Ok(Time::new(self.0))
        } else {
            Err(())
        }
    }

    // pub fn range_to_end(&self, end: Time<G>) -> TimeRangeTo<G> {
    //     TimeRangeTo {
    //         current: *self,
    //         end: end,
    //     }
    // }
}

impl<G> Sequence for Time<G>
where
    G: StaticGranularity,
{
    fn distance_from(&self, offset: &Time<G>) -> i64 {
        (self.0 - offset.0) / G::unit_duration()
    }
}

// pub struct TimeRangeTo<G> {
//     current: Time<G>,
//     end: Time<G>,
// }

impl<G> Into<DateTime<Utc>> for Time<G> {
    fn into(self) -> DateTime<Utc> {
        Utc.timestamp(self.0, 0)
    }
}

// impl<G> std::iter::Iterator for TimeRangeTo<G>
// where
//     G: Granularity + Copy + Ord,
// {
//     type Item = Time<G>;
//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.current + 1;
//         if next >= self.end {
//             None
//         } else {
//             self.current = next;
//             Some(next)
//         }
//     }
// }

impl<G> Add<i64> for Time<G>
where
    G: StaticGranularity,
{
    type Output = Time<G>;
    fn add(self, other: i64) -> Self::Output {
        Time::new(self.0 + G::unit_duration() * other)
    }
}

impl<G> Sub<i64> for Time<G>
where
    G: StaticGranularity,
{
    type Output = Time<G>;
    fn sub(self, other: i64) -> Self::Output {
        Time::new(self.0 - G::unit_duration() * other)
    }
}

pub mod ffi {
    use super::*;
    use crate::granularity::ffi::Var;

    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
    pub struct GTime<G>(i64, G);
    impl GTime<Var> {
        pub fn new(t: i64, g: Var) -> Self {
            debug_assert!(g.is_valid(t));
            GTime(t, g)
        }

        pub fn timestamp(&self) -> i64 {
            self.0
        }

        pub fn granularity(&self) -> Var {
            self.1
        }

        pub fn try_into(self, g2: Var) -> Result<GTime<Var>, ()> {
            if g2.is_valid(self.0) {
                Ok(GTime::new(self.0, g2))
            } else {
                Err(())
            }
        }

        // pub fn range_to_end(&self, end: GTime<Var>) -> TimeRangeTo<Var> {
        //     TimeRangeTo {
        //         current: *self,
        //         end: end,
        //     }
        // }
    }

    impl Sequence for GTime<Var> {
        fn distance_from(&self, offset: &GTime<Var>) -> i64 {
            (self.0 - offset.0) / self.1.unit_duration()
        }
    }

    impl Add<i64> for GTime<Var> {
        type Output = GTime<Var>;
        fn add(self, other: i64) -> Self::Output {
            GTime::new(self.0 + self.1.unit_duration() * other, self.1)
        }
    }

    impl Sub<i64> for GTime<Var> {
        type Output = GTime<Var>;
        fn sub(self, other: i64) -> Self::Output {
            GTime::new(self.0 - self.1.unit_duration() * other, self.1)
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct CTime {
        time: i64,
        granularity: Var,
    }

    use std::convert::Into;
    impl Into<GTime<Var>> for CTime {
        fn into(self) -> GTime<Var> {
            GTime::new(self.time, self.granularity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ffi::*;
    use super::*;
    use crate::granularity::ffi::*;

    #[test]
    fn test_new_s5_ok() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::<S5>::new(dt.timestamp());
    }

    #[test]
    #[should_panic]
    fn test_new_s5_ng() {
        let dt = "2019-01-01T00:00:01Z".parse::<DateTime<Utc>>().unwrap();
        Time::<S5>::new(dt.timestamp());
    }

    #[test]
    fn test_new_d1_ok() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::<D1>::new(dt.timestamp());
    }

    #[test]
    #[should_panic]
    fn test_new_d1_ng() {
        let dt = "2019-01-01T01:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::<D1>::new(dt.timestamp());
    }

    #[test]
    fn test_new_var_ok() {
        let dt = "2019-01-01T07:00:05Z".parse::<DateTime<Utc>>().unwrap();
        GTime::new(dt.timestamp(), Var::new(7));
    }

    #[test]
    #[should_panic]
    fn test_new_var_ng() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        GTime::new(dt.timestamp(), Var::new(7));
    }

    #[test]
    fn test_add_1() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::<S5>::new(dt.timestamp());
        let result = Utc.timestamp((t + 1).timestamp(), 0);
        let expect = "2019-01-01T00:00:05Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_add_2() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::<S5>::new(dt.timestamp());
        let result = Utc.timestamp((t + 2).timestamp(), 0);
        let expect = "2019-01-01T00:00:10Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_conv_ok() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::<S5>::new(dt.timestamp());
        let result = t.try_into::<D1>();
        let expect = Ok(Time::<D1>::new(dt.timestamp()));
        assert_eq!(result, expect);
    }

    #[test]
    fn test_conv_ng() {
        let dt = "2019-01-01T01:00:05Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::<S5>::new(dt.timestamp());
        let result = t.try_into::<D1>();
        let expect = Err(());
        assert_eq!(result, expect);
    }
}
