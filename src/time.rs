pub trait Granularity: Eq + Ord + Clone + Copy + std::hash::Hash {
    fn unit_duration() -> i64;
}

macro_rules! define_granularity {
    ($t:ident, $d:expr) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
        pub struct $t;
        impl Granularity for $t {
            fn unit_duration() -> i64 {
                $d
            }
        }
    };
}
define_granularity!(S5, 5);
define_granularity!(S10, 10);

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct Time<G>(i64, std::marker::PhantomData<G>);

impl<G: Granularity> Time<G> {
    pub fn new(t: i64) -> Self {
        Time(t, std::marker::PhantomData)
    }

    pub fn timestamp(&self) -> i64 {
        self.0
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
    use chrono::prelude::*;

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
}
