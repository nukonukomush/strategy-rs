use super::*;
use crate::library::lru_cache::LRUCache;
use std::cell::RefCell;

pub struct CountContinuousSameValues<G, V, I> {
    source: I,
    cache: RefCell<LRUCache<Time<G>, Option<V>>>,
    phantom: std::marker::PhantomData<G>,
}

impl<G, V, I> CountContinuousSameValues<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy,
    V: Clone,
    // I: FuncIndicator<G, Option<V>>,
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
            phantom: std::marker::PhantomData,
        }
    }

    // fn get_from_cache(&self, time: Time<G>) -> MaybeValue<Option<V>> {
    //     let maybe = self.cache.borrow_mut().get(&time).map(|v| v.clone());
    //     match maybe {
    //         Some(v) => MaybeValue::Value(v),
    //         None => match self.source.value(time) {
    //             MaybeValue::Value(v) => {
    //                 self.cache.borrow_mut().insert(time, v.clone());
    //                 MaybeValue::Value(v)
    //             }
    //             MaybeValue::OutOfRange => MaybeValue::OutOfRange,
    //         },
    //     }
    // }
}

impl<G, V, I> Indicator<G, i32> for CountContinuousSameValues<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy + Debug,
    V: Clone + Debug,
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
    V: Clone + Debug + PartialEq,
    I: FuncIndicator<G, V>,
{
    // TODO: cache
    fn value(&self, time: Time<G>) -> MaybeValue<i32> {
        let value_current = self.source.value(time);
        if value_current == MaybeValue::OutOfRange {
            MaybeValue::OutOfRange
        } else {
            let value_prev = self.source.value(time - 1);
            if value_prev == value_current {
                let count_prev = try_value!(self.value(time - 1));
                MaybeValue::Value(count_prev + 1)
            } else {
                MaybeValue::Value(1)
            }
        }
    }
    // fn value(&self, time: Time<G>) -> MaybeValue<i32> {
    //     let mut t = time;
    //     let mut value = try_value!(self.get_from_cache(t));
    //     while value.is_none() {
    //         t = t - 1;
    //         value = try_value!(self.get_from_cache(t));
    //     }
    //     self.cache.borrow_mut().insert(time, value.clone());
    //     match value {
    //         Some(v) => MaybeValue::Value(v),
    //         None => MaybeValue::OutOfRange,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MaybeValue::*;
    use crate::vec::*;

    #[test]
    fn test_count() {
        let offset = Time::new(0, S5);
        let source = VecIndicator::new(offset, vec![1.0, 1.0, 1.0, 2.0, 3.0, 3.0]);
        let expect = vec![
            Value(1),
            Value(2),
            Value(3),
            Value(1),
            Value(1),
            Value(2),
        ];

        let count = CountContinuousSameValues::new(source, 10);
        let result = (0..6).map(|i| count.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
