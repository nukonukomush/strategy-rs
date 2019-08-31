use super::*;
use crate::library::lru_cache::LRUCache;
use std::cell::RefCell;

pub struct CountContinuousSameValues<G, V, I> {
    source: I,
    cache: RefCell<LRUCache<Time<G>, i32>>,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V>,
}

impl<G, V, I> CountContinuousSameValues<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy,
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }

    fn get_cache(&self, time: Time<G>) -> Option<i32> {
        self.cache.borrow_mut().get(&time).map(|v| v.clone())
    }

    fn set_cache(&self, time: Time<G>, value: i32) {
        self.cache.borrow_mut().insert(time, value);
    }
}

impl<G, V, I> Indicator<G, i32> for CountContinuousSameValues<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy + Debug,
    I: Indicator<G, V>,
{
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

use std::fmt::Debug;
impl<G, V, I> FuncIndicator<G, i32> for CountContinuousSameValues<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy + Debug,
    V: PartialEq,
    I: FuncIndicator<G, V>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<i32> {
        let cache = self.get_cache(time);
        match cache {
            Some(count) => MaybeValue::Value(count),
            None => {
                let value_current = self.source.value(time);
                if value_current == MaybeValue::OutOfRange {
                    MaybeValue::OutOfRange
                } else {
                    let value_prev = self.source.value(time - 1);
                    let count = if value_prev == value_current {
                        let count_prev = try_value!(self.value(time - 1));
                        count_prev + 1
                    } else {
                        1
                    };
                    self.set_cache(time, count);
                    MaybeValue::Value(count)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;

    #[test]
    fn test_count() {
        let offset = Time::new(0, S5);
        let source = VecIndicator::new(offset, vec![1.0, 1.0, 1.0, 2.0, 3.0, 3.0]);
        let expect = vec![Value(1), Value(2), Value(3), Value(1), Value(1), Value(2)];

        let count = CountContinuousSameValues::new(source, 10);
        let result = (0..6).map(|i| count.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
