use super::zone::*;
use crate::indicator::*;
use crate::seq::*;
use MaybeFixed::*;
use MaybeInRange::*;


use crate::library::lru_cache::LRUCache;
use std::cell::RefCell;
pub struct IsEntriedInZone<S, I1, I2, I3> {
    outermost_zone: I1,
    up_down: I2,
    up_down_count: I3,
    cache: RefCell<LRUCache<S, bool>>,
}

impl<S, I1, I2, I3> IsEntriedInZone<S, I1, I2, I3>
where
    S: Sequence,
    I1: Indicator<Seq = S, Val = ZoneId>,
    I2: Indicator<Seq = S, Val = UpDown>,
    I3: Indicator<Seq = S, Val = i32>,
{
    pub fn new(outermost_zone: I1, up_down: I2, up_down_count: I3, capacity: usize) -> Self {
        Self {
            outermost_zone: outermost_zone,
            up_down: up_down,
            up_down_count: up_down_count,
            cache: RefCell::new(LRUCache::new(capacity)),
        }
    }

    fn get_cache(&self, seq: S) -> Option<bool> {
        self.cache.borrow_mut().get(&seq).map(|v| v.clone())
    }

    fn set_cache(&self, seq: S, value: bool) {
        self.cache.borrow_mut().insert(seq, value);
    }
}

impl<S, I1, I2, I3> Indicator for IsEntriedInZone<S, I1, I2, I3>
where
    S: Sequence,
    I1: Indicator<Seq = S, Val = ZoneId>,
    I2: Indicator<Seq = S, Val = UpDown>,
    I3: Indicator<Seq = S, Val = i32>,
{
    type Seq = S;
    type Val = bool;
}

impl<S, I1, I2, I3> FuncIndicator for IsEntriedInZone<S, I1, I2, I3>
where
    S: Sequence,
    I1: FuncIndicator<Seq = S, Val = ZoneId>,
    I2: FuncIndicator<Seq = S, Val = UpDown>,
    I3: FuncIndicator<Seq = S, Val = i32>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let cache = self.get_cache(seq);
        match cache {
            Some(is_entried) => return Fixed(InRange(is_entried)),
            None => (),
        }
        let zone = try_value!(self.outermost_zone.value(seq));
        if zone == ZoneId(0) {
            // ゾーン 0 ならリセット
            self.set_cache(seq, false);
            Fixed(InRange(false))
        } else {
            let prev_zone = try_value!(self.outermost_zone.value(seq - 1));
            if prev_zone != zone {
                // ゾーン更新されたらリセット
                self.set_cache(seq, false);
                Fixed(InRange(false))
            } else {
                match self.value(seq - 1) {
                    Fixed(InRange(prev_is_entried)) => {
                        let up_down = try_value!(self.up_down.value(seq));
                        let up_down_count = try_value!(self.up_down_count.value(seq));
                        let is_entried = if !prev_is_entried && zone.is_inverse(up_down) {
                            // ゾーン 3,4,5 は 2 ticks
                            if zone.is_outer_than(ZoneId(2)) || zone.is_outer_than(ZoneId(-2)) {
                                up_down_count >= 2
                            } else {
                                up_down_count >= 1
                            }
                        } else {
                            prev_is_entried
                        };
                        self.set_cache(seq, is_entried);
                        Fixed(InRange(is_entried))
                    }
                    other => other,
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::seq::*;
    use crate::time::*;
    use crate::vec::*;

    #[test]
    fn test_entried() {
        let offset = Time::<S5>::new(0);
        let source = vec![
            (ZoneId(0), UpDown::Up, 1, false),
            (ZoneId(1), UpDown::Up, 2, false),
            (ZoneId(1), UpDown::Down, 1, true),
            (ZoneId(0), UpDown::Up, 1, false),
            (ZoneId(0), UpDown::Up, 2, false),
            (ZoneId(-1), UpDown::Down, 1, false),
            (ZoneId(-1), UpDown::Down, 2, false),
            (ZoneId(-1), UpDown::Down, 3, false),
            (ZoneId(-2), UpDown::Down, 4, false),
            (ZoneId(-2), UpDown::Up, 1, true),
            (ZoneId(-2), UpDown::Down, 1, true),
            (ZoneId(-3), UpDown::Down, 2, false),
            (ZoneId(-3), UpDown::Up, 1, false),
            (ZoneId(-3), UpDown::Up, 2, true),
            (ZoneId(-3), UpDown::Up, 3, true),
            (ZoneId(-3), UpDown::Up, 4, true),
            (ZoneId(0), UpDown::Up, 5, false),
        ];
        let expect = source
            .iter()
            .map(|i| Fixed(InRange(i.3.clone())))
            .collect::<Vec<_>>();

        let zone = VecIndicator::new(
            offset,
            source.iter().map(|i| i.0.clone()).collect::<Vec<_>>(),
        );
        let up_down = VecIndicator::new(
            offset,
            source.iter().map(|i| i.1.clone()).collect::<Vec<_>>(),
        );
        let up_down_count = VecIndicator::new(
            offset,
            source.iter().map(|i| i.2.clone()).collect::<Vec<_>>(),
        );
        let is_entried = IsEntriedInZone::new(zone, up_down, up_down_count, 20);

        let result = (0..17)
            .map(|i| is_entried.value(offset + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
