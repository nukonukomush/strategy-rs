pub trait Granularity: Eq + Ord + Clone + Copy + std::hash::Hash + std::fmt::Debug {
    fn unit_duration() -> i64;
    fn is_valid(t: i64) -> bool;
}

macro_rules! define_granularity {
    ($t:ident, $d:expr, $v:expr) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
        pub struct $t;
        impl Granularity for $t {
            fn unit_duration() -> i64 {
                $d
            }
            fn is_valid(t: i64) -> bool {
                $v(t)
            }
        }
    };
}
use chrono::prelude::*;
define_granularity!(S5, 5, |t| {
    let dt = Utc.timestamp(t, 0);
    dt.second() % 5 == 0
});
define_granularity!(S10, 10, |t| {
    let dt = Utc.timestamp(t, 0);
    dt.second() % 10 == 0
});
define_granularity!(D1, 60 * 60 * 24, |t| {
    let dt = Utc.timestamp(t, 0);
    dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0
});

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct Time<G>(i64, std::marker::PhantomData<G>);

impl<G: Granularity> Time<G> {
    pub fn new(t: i64) -> Self {
        debug_assert!(G::is_valid(t));
        Time(t, std::marker::PhantomData)
    }

    pub fn timestamp(&self) -> i64 {
        self.0
    }

    pub fn try_into<G2: Granularity>(self) -> Result<Time<G2>, ()> {
        if G2::is_valid(self.0) {
            Ok(Time::<G2>::new(self.0))
        } else {
            Err(())
        }
    }
}

use std::ops::Add;
impl<G: Granularity> Add<i64> for Time<G> {
    type Output = Time<G>;
    fn add(self, other: i64) -> Self::Output {
        Time::new(self.0 + G::unit_duration() * other)
    }
}

use std::ops::Sub;
impl<G: Granularity> Sub<i64> for Time<G> {
    type Output = Time<G>;
    fn sub(self, other: i64) -> Self::Output {
        Time::new(self.0 - G::unit_duration() * other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
