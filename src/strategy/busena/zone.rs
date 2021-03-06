use crate::granularity::*;
use crate::indicator::*;
use crate::seq::*;
use crate::time::*;
use std::cell::RefCell;
use std::rc::Rc;
use MaybeFixed::*;
use MaybeInRange::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ZoneId(pub i32);

impl ZoneId {
    pub fn is_outer_than(&self, other: Self) -> bool {
        if other.0 == 0 {
            self.0 != 0
        } else if other.0 > 0 {
            self.0 > other.0
        } else if other.0 < 0 {
            self.0 < other.0
        } else {
            panic!("");
        }
    }

    pub fn is_crossed_zero(&self, other: Self) -> bool {
        self.0 * other.0 < 0
    }

    pub fn is_inverse(&self, up_down: UpDown) -> bool {
        if self.0 == 0 {
            false
        } else if self.0 > 0 {
            up_down == UpDown::Down
        } else if self.0 < 0 {
            up_down == UpDown::Up
        } else {
            panic!("");
        }
    }
}

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

use crate::library::lru_cache::LRUCache;
pub struct OutermostZone<S, I> {
    zone: I,
    cache: RefCell<LRUCache<S, ZoneId>>,
}

impl<S, I> OutermostZone<S, I>
where
    S: Sequence,
{
    pub fn new(zone: I, capacity: usize) -> Self {
        Self {
            zone: zone,
            cache: RefCell::new(LRUCache::new(capacity)),
        }
    }

    fn get_cache(&self, seq: S) -> Option<ZoneId> {
        self.cache.borrow_mut().get(&seq).map(|v| v.clone())
    }

    fn set_cache(&self, seq: S, value: ZoneId) {
        self.cache.borrow_mut().insert(seq, value);
    }
}

impl<S, I> Indicator for OutermostZone<S, I>
where
    S: Sequence,
    I: Indicator<Seq = S, Val = ZoneId>,
{
    type Seq = S;
    type Val = ZoneId;
}

impl<S, I> FuncIndicator for OutermostZone<S, I>
where
    S: Sequence,
    I: FuncIndicator<Seq = S, Val = ZoneId>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let cache = self.get_cache(seq);
        match cache {
            Some(outermost_zone) => return Fixed(InRange(outermost_zone)),
            None => (),
        }

        let zone = try_value!(self.zone.value(seq));
        if zone == ZoneId(0) {
            // ????????? 0 ??????????????????
            self.set_cache(seq, ZoneId(0));
            Fixed(InRange(ZoneId(0)))
        } else {
            match self.value(seq - 1) {
                Fixed(InRange(prev_outermost_zone)) => {
                    if zone.is_crossed_zero(prev_outermost_zone) {
                        // 0 ???????????????????????????
                        self.set_cache(seq, zone);
                        Fixed(InRange(zone))
                    } else if zone.is_outer_than(prev_outermost_zone) {
                        // ??????????????????????????????????????????
                        self.set_cache(seq, zone);
                        Fixed(InRange(zone))
                    } else {
                        Fixed(InRange(prev_outermost_zone))
                    }
                }
                _ => {
                    self.set_cache(seq, zone);
                    Fixed(InRange(zone))
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpDown {
    Neutral,
    Up,
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
            UpDown::Neutral
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

// TODO: ??????????????????
//
#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, ZoneId, Zone<FuncIndicatorPtr<S, V>, FuncIndicatorPtr<S, V>>>;

    // pub unsafe fn new<S, CS, V, CV>(
    //     price: *mut FuncIndicatorPtr<S, V>,
    //     positive_lines: *const *mut FuncIndicatorPtr<S, V>,
    //     positive_lines_length: i32,
    //     negative_lines: *const *mut FuncIndicatorPtr<S, V>,
    //     negative_lines_length: i32,
    // ) -> IPtr<S, V>
    // where
    //     S: Sequence + 'static,
    //     CS: Into<S>,
    //     V: Clone + std::fmt::Debug + 'static,
    //     CV: Into<V>,
    // {
    //     let price = (*price).clone();
    //     let positive_lines: &[*mut FuncIndicatorPtr<S, V>] =
    //         std::slice::from_raw_parts(positive_lines, positive_lines_length as usize);
    //     let positive_lines = positive_lines
    //         .iter()
    //         .map(|cv| cv.clone().into())
    //         .collect::<Vec<_>>();
    //     let negative_lines: &[*mut FuncIndicatorPtr<S, V>] =
    //         std::slice::from_raw_parts(negative_lines, negative_lines_length as usize);
    //     let negative_lines = negative_lines
    //         .iter()
    //         .map(|cv| cv.clone().into())
    //         .collect::<Vec<_>>();
    //     let ptr = Zone::new(price, positive_lines, negative_lines).into_sync_ptr();
    //     Ptr {
    //         b_ptr: Box::into_raw(Box::new(ptr.clone())),
    //         f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
    //     }
    // }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                price: *mut FuncIndicatorPtr<$s, $v>,
                positive_lines: *const *mut FuncIndicatorPtr<$s, $v>,
                positive_lines_length: i32,
                negative_lines: *const *mut FuncIndicatorPtr<$s, $v>,
                negative_lines_length: i32,
            ) -> IPtr<$s, $v> {
                // new::<$s, $cs, $v, $cv>(
                //     price,
                //     positive_lines,
                //     positive_lines_length,
                //     negative_lines,
                //     negative_lines_length,
                // )
                let price = (*price).clone();
                let positive_lines: &[*mut FuncIndicatorPtr<$s, $v>] =
                    std::slice::from_raw_parts(positive_lines, positive_lines_length as usize);
                let positive_lines = positive_lines
                    .iter()
                    .map(|cv| (**cv).clone())
                    .collect::<Vec<FuncIndicatorPtr<_, _>>>();
                let negative_lines: &[*mut FuncIndicatorPtr<$s, $v>] =
                    std::slice::from_raw_parts(negative_lines, negative_lines_length as usize);
                let negative_lines = negative_lines
                    .iter()
                    .map(|cv| (**cv).clone())
                    .collect::<Vec<FuncIndicatorPtr<_, _>>>();
                let ptr = Zone::new(price, positive_lines, negative_lines).into_sync_ptr();
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }
        };
    }

    define_new!(TickId, i64, f64, f64, zone_new_tick_id_f64);

    define_destroy!(IPtr<TickId, f64>, zone_destroy_tick_id_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::seq::*;
    use crate::vec::*;

    #[test]
    fn test_zone() {
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
            Fixed(InRange(Neutral)),
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

    // // TODO: ????????????????????????
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

    #[test]
    fn test_outermost_zone() {
        let offset = Time::<S5>::new(0);
        let source = vec![
            ZoneId(0),
            ZoneId(1),
            ZoneId(0),
            ZoneId(2),
            ZoneId(1),
            ZoneId(2),
            ZoneId(3),
            ZoneId(-4),
        ];
        let expect = vec![
            Fixed(InRange(ZoneId(0))),
            Fixed(InRange(ZoneId(1))),
            Fixed(InRange(ZoneId(0))),
            Fixed(InRange(ZoneId(2))),
            Fixed(InRange(ZoneId(2))),
            Fixed(InRange(ZoneId(2))),
            Fixed(InRange(ZoneId(3))),
            Fixed(InRange(ZoneId(-4))),
        ];

        let source = VecIndicator::new(offset, source);
        let outermost_zone = OutermostZone::new(source, 20);

        let result = (0..8)
            .map(|i| outermost_zone.value(offset + i))
            .collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
