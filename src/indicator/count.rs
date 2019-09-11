use super::*;
use crate::library::lru_cache::LRUCache;
use std::cell::RefCell;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct CountContinuousSameValues<S, I> {
    source: I,
    cache: RefCell<LRUCache<S, i32>>,
}

impl<S, I> CountContinuousSameValues<S, I>
where
    S: Sequence,
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
        }
    }

    fn get_cache(&self, seq: S) -> Option<i32> {
        self.cache.borrow_mut().get(&seq).map(|v| v.clone())
    }

    fn set_cache(&self, seq: S, value: i32) {
        self.cache.borrow_mut().insert(seq, value);
    }
}

impl<S, I> Indicator for CountContinuousSameValues<S, I>
where
    S: Sequence,
    I: Indicator<Seq = S>,
{
    type Seq = I::Seq;
    type Val = i32;
}

// use std::fmt::Debug;
impl<S, V, I> FuncIndicator for CountContinuousSameValues<S, I>
where
    S: Sequence,
    V: PartialEq + std::fmt::Debug,
    I: FuncIndicator<Seq = S, Val = V>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        let cache = self.get_cache(seq);
        match cache {
            Some(count) => Fixed(InRange(count)),
            None => {
                let value_current = self.source.value(seq);
                if value_current == NotFixed {
                    NotFixed
                } else if value_current == Fixed(OutOfRange) {
                    Fixed(OutOfRange)
                } else {
                    let value_prev = self.source.value(seq - 1);
                    let count = if value_prev == value_current {
                        let count_prev = try_value!(self.value(seq - 1));
                        count_prev + 1
                    } else {
                        1
                    };
                    self.set_cache(seq, count);
                    Fixed(InRange(count))
                }
                // if value_current == MaybeValue::OutOfRange {
                //     MaybeValue::OutOfRange
                // } else {
                //     let value_prev = self.source.value(seq - 1);
                //     let count = if value_prev == value_current {
                //         let count_prev = try_value!(self.value(seq - 1));
                //         count_prev + 1
                //     } else {
                //         1
                //     };
                //     self.set_cache(seq, count);
                //     MaybeValue::Value(count)
                // }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_count() {
        let offset = Time::<S5>::new(0);
        let source = VecIndicator::new(offset, vec![1.0, 1.0, 1.0, 2.0, 3.0, 3.0]);
        let expect = vec![
            Fixed(InRange(1)),
            Fixed(InRange(2)),
            Fixed(InRange(3)),
            Fixed(InRange(1)),
            Fixed(InRange(1)),
            Fixed(InRange(2)),
        ];

        let count = CountContinuousSameValues::new(source, 10);
        let result = (0..6).map(|i| count.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
