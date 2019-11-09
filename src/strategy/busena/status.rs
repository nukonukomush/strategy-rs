use crate::indicator::*;
use super::zone::*;

#[derive(PartialEq, Eq, Debug)]
pub struct Status {
    outermost_zone: ZoneId,
    is_entried: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            outermost_zone: ZoneId(0),
            is_entried: false,
        }
    }

    pub fn is_entried(&self) -> bool {
        self.is_entried
    }

    pub fn outermost_zone(&self) -> ZoneId {
        self.outermost_zone
    }

    pub fn update(&mut self, zone: ZoneId, up_down: UpDown, up_down_count: usize) {
        if zone == ZoneId(0) {
            // ゾーン 0 なら全てリセット
            self.outermost_zone = ZoneId(0);
            self.is_entried = false;
        } else if zone.is_outer_than(self.outermost_zone) {
            // 外側ゾーンに移行したなら更新 & エントリーフラグリセット
            self.outermost_zone = zone;
            self.is_entried = false;
        } else if self.outermost_zone == zone {
            // ゾーンが同じ場合
            // 未エントリーの場合、逆行したらエントリー
            if !self.is_entried && self.outermost_zone.is_inverse(up_down) {
                // ゾーン 3,4,5 は 2 ticks
                println!("{:?}", self.outermost_zone.is_outer_than(ZoneId(2)));
                if self.outermost_zone.is_outer_than(ZoneId(2))
                    || self.outermost_zone.is_outer_than(ZoneId(-2))
                {
                    if up_down_count >= 2 {
                        println!("aa {:?}", self.outermost_zone);
                        self.is_entried = true;
                    }
                } else {
                    if up_down_count >= 1 {
                        println!("bb {:?}", self.outermost_zone);
                        self.is_entried = true;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 0 => 0
    #[test]
    fn test_status_0to0() {
        let mut status = Status::new();
        status.update(ZoneId(0), UpDown::Neutral, 0);
        let expect = Status {
            outermost_zone: ZoneId(0),
            is_entried: false,
        };
        assert_eq!(status, expect);
    }

    // 0 => 1
    #[test]
    fn test_status_0to1() {
        let mut status = Status {
            outermost_zone: ZoneId(0),
            is_entried: false,
        };
        let expect = Status {
            outermost_zone: ZoneId(1),
            is_entried: false,
        };
        status.update(ZoneId(1), UpDown::Up, 1);
        assert_eq!(status, expect);
    }

    // 1 => 2
    #[test]
    fn test_status_1to2() {
        let mut status = Status {
            outermost_zone: ZoneId(1),
            is_entried: false,
        };
        let expect = Status {
            outermost_zone: ZoneId(2),
            is_entried: false,
        };
        status.update(ZoneId(2), UpDown::Up, 1);
        assert_eq!(status, expect);
    }

    // 0 => -2
    #[test]
    fn test_status_0tom2() {
        let mut status = Status {
            outermost_zone: ZoneId(0),
            is_entried: true,
        };
        let expect = Status {
            outermost_zone: ZoneId(-2),
            is_entried: false,
        };
        status.update(ZoneId(-2), UpDown::Down, 1);
        assert_eq!(status, expect);
    }

    // 2 => 1
    #[test]
    fn test_status_2to1() {
        let mut status = Status {
            outermost_zone: ZoneId(2),
            is_entried: true,
        };
        let expect = Status {
            outermost_zone: ZoneId(2),
            is_entried: true,
        };
        status.update(ZoneId(1), UpDown::Down, 1);
        assert_eq!(status, expect);
    }

    // 2 => 0
    #[test]
    fn test_status_2to0() {
        let mut status = Status {
            outermost_zone: ZoneId(2),
            is_entried: true,
        };
        let expect = Status {
            outermost_zone: ZoneId(0),
            is_entried: false,
        };
        status.update(ZoneId(0), UpDown::Down, 1);
        assert_eq!(status, expect);
    }

    // -2 => -3
    #[test]
    fn test_status_m2tom3() {
        let mut status = Status {
            outermost_zone: ZoneId(-2),
            is_entried: true,
        };
        let expect = Status {
            outermost_zone: ZoneId(-3),
            is_entried: false,
        };
        status.update(ZoneId(-3), UpDown::Down, 1);
        assert_eq!(status, expect);
    }

    // 2 => 2 entry
    #[test]
    fn test_status_2to2_entry() {
        let mut status = Status {
            outermost_zone: ZoneId(2),
            is_entried: false,
        };
        let expect = Status {
            outermost_zone: ZoneId(2),
            is_entried: true,
        };
        status.update(ZoneId(2), UpDown::Down, 1);
        assert_eq!(status, expect);
    }

    // 3 => 3 entry
    #[test]
    fn test_status_3to3_entry_1() {
        let mut status = Status {
            outermost_zone: ZoneId(3),
            is_entried: false,
        };
        let expect = Status {
            outermost_zone: ZoneId(3),
            is_entried: false,
        };
        status.update(ZoneId(3), UpDown::Down, 1);
        assert_eq!(status, expect);
    }

    // 3 => 3 entry
    #[test]
    fn test_status_3to3_entry_2() {
        let mut status = Status {
            outermost_zone: ZoneId(3),
            is_entried: false,
        };
        let expect = Status {
            outermost_zone: ZoneId(3),
            is_entried: true,
        };
        status.update(ZoneId(3), UpDown::Down, 2);
        assert_eq!(status, expect);
    }
}
