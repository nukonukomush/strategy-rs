// pub trait Granularity: Eq + Ord + Clone + Copy + std::hash::Hash + std::fmt::Debug {
pub trait Granularity {
    fn unit_duration(&self) -> i64;
    fn is_valid(&self, t: i64) -> bool;
}

macro_rules! define_granularity {
    ($t:ident, $d:expr, $v:expr) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
        pub struct $t;
        impl Granularity for $t {
            fn unit_duration(&self) -> i64 {
                $d
            }
            fn is_valid(&self, t: i64) -> bool {
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
pub struct VarGranularity(i64);
impl VarGranularity {
    pub fn new(d: i64) -> Self {
        debug_assert_ne!(d, 0);
        VarGranularity(d)
    }
}
impl Granularity for VarGranularity {
    fn unit_duration(&self) -> i64 {
        self.0
    }
    fn is_valid(&self, t: i64) -> bool {
        println!("{}", t % self.0);
        t % self.0 == 0
    }
}

// #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
// pub struct GranularityPtr(Box<dyn Granularity>);
// impl Granularity for DynamicGranularity {
//     fn unit_duration(&self) -> i64 {
//         self.0.unit_duration()
//     }
//     fn is_valid(&self, t: i64) -> bool {
//         self.0.is_valid(t)
//     }
// }

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct Time<G>(i64, G);

impl<G: Granularity + Copy> Time<G> {
    pub fn new(t: i64, g: G) -> Self {
        debug_assert!(g.is_valid(t));
        Time(t, g)
    }

    pub fn timestamp(&self) -> i64 {
        self.0
    }

    pub fn granularity(&self) -> G {
        self.1
    }

    pub fn try_into<G2: Granularity + Copy>(self, g: G2) -> Result<Time<G2>, ()> {
        if g.is_valid(self.0) {
            Ok(Time::new(self.0, g))
        } else {
            Err(())
        }
    }
}

use std::ops::Add;
impl<G: Granularity + Copy> Add<i64> for Time<G> {
    type Output = Time<G>;
    fn add(self, other: i64) -> Self::Output {
        Time::new(self.0 + self.1.unit_duration() * other, self.1)
    }
}

use std::ops::Sub;
impl<G: Granularity + Copy> Sub<i64> for Time<G> {
    type Output = Time<G>;
    fn sub(self, other: i64) -> Self::Output {
        Time::new(self.0 - self.1.unit_duration() * other, self.1)
    }
}

pub mod ffi {
    use super::*;

    #[repr(C)]
    pub struct CTime {
        time: i64,
        granularity: VarGranularity,
    }

    use std::convert::Into;
    impl Into<Time<VarGranularity>> for CTime {
        fn into(self) -> Time<VarGranularity> {
            Time::new(self.time, self.granularity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_s5_ok() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::new(dt.timestamp(), S5);
    }

    #[test]
    #[should_panic]
    fn test_new_s5_ng() {
        let dt = "2019-01-01T00:00:01Z".parse::<DateTime<Utc>>().unwrap();
        Time::new(dt.timestamp(), S5);
    }

    #[test]
    fn test_new_d1_ok() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::new(dt.timestamp(), D1);
    }

    #[test]
    #[should_panic]
    fn test_new_d1_ng() {
        let dt = "2019-01-01T01:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::new(dt.timestamp(), D1);
    }

    #[test]
    fn test_new_var_ok() {
        let dt = "2019-01-01T07:00:05Z".parse::<DateTime<Utc>>().unwrap();
        Time::new(dt.timestamp(), VarGranularity::new(7));
    }

    #[test]
    #[should_panic]
    fn test_new_var_ng() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        Time::new(dt.timestamp(), VarGranularity::new(7));
    }

    #[test]
    fn test_add_1() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::new(dt.timestamp(), S5);
        let result = Utc.timestamp((t + 1).timestamp(), 0);
        let expect = "2019-01-01T00:00:05Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_add_2() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::new(dt.timestamp(), S5);
        let result = Utc.timestamp((t + 2).timestamp(), 0);
        let expect = "2019-01-01T00:00:10Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_conv_ok() {
        let dt = "2019-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::new(dt.timestamp(), S5);
        let result = t.try_into(D1);
        let expect = Ok(Time::new(dt.timestamp(), D1));
        assert_eq!(result, expect);
    }

    #[test]
    fn test_conv_ng() {
        let dt = "2019-01-01T01:00:05Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time::new(dt.timestamp(), S5);
        let result = t.try_into(D1);
        let expect = Err(());
        assert_eq!(result, expect);
    }
}
