use super::*;
use crate::library::lru_cache::LRUCache;
use log::*;
use std::cell::RefCell;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Ema<S, V, I> {
    source: I,
    alpha: f64,
    actual_period: usize,
    cache: RefCell<LRUCache<S, V>>,
}

impl<S, V, I> Ema<S, V, I>
where
    S: Sequence,
    V: Clone + std::fmt::Debug,
{
    pub fn new(source: I, n_period: usize, accuracy: f64, capacity: usize) -> Self {
        let alpha = Self::calc_alpha(n_period);
        let actual_period = Self::calc_actual_period(accuracy, alpha);
        println!("alpha: {:?}", alpha);
        println!("period: {:?}", actual_period);
        Self {
            source: source,
            alpha: alpha,
            actual_period: actual_period,
            cache: RefCell::new(LRUCache::new(capacity)),
        }
    }

    pub fn calc_alpha(n_period: usize) -> f64 {
        2.0 / (n_period as f64 + 1.0)
    }

    pub fn calc_actual_period(accuracy: f64, alpha: f64) -> usize {
        assert!(0.0 < accuracy && accuracy < 1.0);
        let k = (1.0 - accuracy).ln() / (1.0 - alpha).ln();
        k.ceil() as usize
    }

    fn get_cache(&self, seq: S) -> Option<V> {
        self.cache.borrow_mut().get(&seq).map(|v| v.clone())
    }

    fn set_cache(&self, seq: S, value: V) {
        self.cache.borrow_mut().insert(seq, value);
    }
}

impl<S, I> Ema<S, f64, I>
where
    S: Sequence,
    I: FuncIndicator<Seq = S, Val = f64>,
{
    fn value_recursive(&self, seq: S, remain_times: usize) -> MaybeValue<f64> {
        if remain_times == 0 {
            // ここは sma にしてもよい
            // first value の source を注入できるようにする？
            return self.source.value(seq);
        }

        let cache = self.get_cache(seq);
        match cache {
            Some(v) => Fixed(InRange(v)),
            None => self
                .value_recursive(seq - 1, remain_times - 1)
                .zip2(self.source.value(seq))
                .map2(|(prev_ema, src_value)| prev_ema + (src_value - prev_ema) * self.alpha)
                .map2(|v| {
                    self.set_cache(seq, v.clone());
                    v
                }),
        }
    }
}

impl<S, V, I> Indicator for Ema<S, V, I>
where
    S: Sequence,
    V: Clone + std::fmt::Debug,
    I: Indicator<Seq = S, Val = V>,
{
    type Seq = I::Seq;
    type Val = V;
}

impl<S, I> FuncIndicator for Ema<S, f64, I>
where
    S: Sequence,
    I: FuncIndicator<Seq = S, Val = f64>,
{
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.value_recursive(seq, self.actual_period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::vec::*;

    #[test]
    fn test_ema_const_1() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0; 50];
        let expect = [
            vec![Fixed(OutOfRange); 6].as_slice(),
            vec![Fixed(InRange(1.0)); 44].as_slice(),
        ]
        .concat();
        let n_period = 5;
        let accuracy = 0.9;
        let capacity = 100;
        let ema = Ema::new(
            VecIndicator::new(offset, source.clone()),
            n_period,
            accuracy,
            capacity,
        );

        let result = (0..50).map(|i| ema.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_ema_const_2() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0; 50];
        let expect = [
            vec![Fixed(OutOfRange); 24].as_slice(),
            vec![Fixed(InRange(1.0)); 26].as_slice(),
        ]
        .concat();
        let n_period = 20;
        let accuracy = 0.9;
        let capacity = 100;
        let ema = Ema::new(
            VecIndicator::new(offset, source.clone()),
            n_period,
            accuracy,
            capacity,
        );

        let result = (0..50).map(|i| ema.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_ema_const_3() {
        let offset = Time::<S5>::new(0);
        let source = vec![1.0; 100];
        let expect = [
            vec![Fixed(OutOfRange); 70].as_slice(),
            vec![Fixed(InRange(1.0)); 30].as_slice(),
        ]
        .concat();
        let n_period = 20;
        let accuracy = 0.999;
        let capacity = 100;
        let ema = Ema::new(
            VecIndicator::new(offset, source.clone()),
            n_period,
            accuracy,
            capacity,
        );

        let result = (0..100).map(|i| ema.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_ema_1() {
        let offset = Time::<S5>::new(0);
        let source = [vec![1.0; 5].as_slice(), vec![3.0; 5].as_slice()].concat();
        let expect = [
            vec![Fixed(OutOfRange); 4].as_slice(),
            vec![
                Fixed(InRange(1.0)),
                Fixed(InRange(2.0)),
                Fixed(InRange(2.5)),
                Fixed(InRange(2.75)),
                Fixed(InRange(2.875)),
                Fixed(InRange(2.9375)),
            ]
            .as_slice(),
        ]
        .concat();
        let n_period = 3;
        let accuracy = 0.9;
        let capacity = 100;
        let ema = Ema::new(
            VecIndicator::new(offset, source.clone()),
            n_period,
            accuracy,
            capacity,
        );

        let result = (0..10).map(|i| ema.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
