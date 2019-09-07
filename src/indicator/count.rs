use super::*;
use crate::seq::*;
use crate::library::lru_cache::LRUCache;
use std::cell::RefCell;

pub struct CountContinuousSameValues<S, V, I> {
    source: I,
    cache: RefCell<LRUCache<S, i32>>,
    p1: std::marker::PhantomData<S>,
    p2: std::marker::PhantomData<V>,
}

impl<S, V, I> CountContinuousSameValues<S, V, I>
where
    S: Sequence,
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }

    fn get_cache(&self, seq: S) -> Option<i32> {
        self.cache.borrow_mut().get(&seq).map(|v| v.clone())
    }

    fn set_cache(&self, seq: S, value: i32) {
        self.cache.borrow_mut().insert(seq, value);
    }
}

impl<S, V, I> Indicator<S, i32> for CountContinuousSameValues<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy + Debug,
    S:Sequence,
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

use std::fmt::Debug;
impl<S, V, I> FuncIndicator<S, i32> for CountContinuousSameValues<S, V, I>
where
    S: Sequence,
    V: PartialEq,
    I: FuncIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<i32> {
        let cache = self.get_cache(seq);
        match cache {
            Some(count) => MaybeValue::Value(count),
            None => {
                let value_current = self.source.value(seq);
                if value_current == MaybeValue::OutOfRange {
                    MaybeValue::OutOfRange
                } else {
                    let value_prev = self.source.value(seq - 1);
                    let count = if value_prev == value_current {
                        let count_prev = try_value!(self.value(seq - 1));
                        count_prev + 1
                    } else {
                        1
                    };
                    self.set_cache(seq, count);
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
    use crate::granularity::*;

    #[test]
    fn test_count() {
        let offset = Time::<S5>::new(0);
        let source = VecIndicator::new(offset, vec![1.0, 1.0, 1.0, 2.0, 3.0, 3.0]);
        let expect = vec![Value(1), Value(2), Value(3), Value(1), Value(1), Value(2)];

        let count = CountContinuousSameValues::new(source, 10);
        let result = (0..6).map(|i| count.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
