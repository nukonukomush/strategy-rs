use chrono::prelude::*;

pub trait Granularity: Eq + Ord + Clone + Copy + std::hash::Hash + std::fmt::Debug {}

pub trait StaticGranularity: Granularity {
    fn unit_duration() -> i64;
    fn is_valid(t: i64) -> bool;
}

macro_rules! define_static_granularity {
    ($t:ident, $d:expr, $v:expr) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
        pub struct $t;
        impl Granularity for $t {}
        impl StaticGranularity for $t {
            fn unit_duration() -> i64 {
                $d
            }
            fn is_valid(t: i64) -> bool {
                $v(t)
            }
        }
    };
}
define_static_granularity!(S5, 5, |t| {
    let dt = Utc.timestamp(t, 0);
    dt.second() % 5 == 0
});
define_static_granularity!(S10, 10, |t| {
    let dt = Utc.timestamp(t, 0);
    dt.second() % 10 == 0
});
define_static_granularity!(D1, 60 * 60 * 24, |t| {
    let dt = Utc.timestamp(t, 0);
    dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0
});

pub mod ffi {
    use super::*;

    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
    pub struct Var(i64);
    impl Var {
        pub fn new(d: i64) -> Self {
            debug_assert_ne!(d, 0);
            Var(d)
        }

        pub fn unit_duration(&self) -> i64 {
            self.0
        }

        pub fn is_valid(&self, t: i64) -> bool {
            t % self.0 == 0
        }
    }
    impl Granularity for Var {}
}
