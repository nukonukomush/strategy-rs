use super::*;
use crate::library::lru_cache::LRUCache as Cache;
use crate::seq::*;
use crate::time::*;
use crate::*;
use std::cell::RefCell;

pub struct LRUCache<S, V, I> {
    source: I,
    cache: RefCell<Cache<S, V>>,
}

impl<S, V, I> LRUCache<S, V, I>
where
    S: Sequence,
    V: Clone,
    I: Indicator<S, V>,
{
    pub fn new(capacity: usize, source: I) -> Self {
        Self {
            source: source,
            cache: RefCell::new(Cache::new(capacity)),
        }
    }
}

impl<S, V, I> Indicator<S, V> for LRUCache<S, V, I>
where
    S: Sequence,
    V: Clone,
    I: Indicator<S, V>,
{
}

impl<S, V, I> FuncIndicator<S, V> for LRUCache<S, V, I>
where
    S: Sequence,
    V: Clone,
    I: FuncIndicator<S, V>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        let maybe = self.cache.borrow_mut().get(&seq).map(|v| v.clone());
        match maybe {
            Some(v) => MaybeValue::Value(v),
            None => match self.source.value(seq) {
                MaybeValue::Value(v) => {
                    self.cache.borrow_mut().insert(seq, v.clone());
                    MaybeValue::Value(v)
                }
                MaybeValue::OutOfRange => MaybeValue::OutOfRange,
            },
        }
    }
}

// #[cfg(ffi)]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, LRUCache<S, V, FuncIndicatorPtr<S, V>>>;

    pub unsafe fn new<S, CS, V, CV>(
        capacity: c_int,
        source: *mut FuncIndicatorPtr<S, V>,
    ) -> IPtr<S, V>
    where
        S: Sequence + 'static,
        CS: Into<S>,
        V: Clone + 'static,
        CV: Into<V>,
    {
        let source = (*source).clone();
        let ptr = Rc::new(RefCell::new(LRUCache::new(capacity as usize, source)));
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
        }
    }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                capacity: c_int,
                source: *mut FuncIndicatorPtr<$s, $v>,
            ) -> IPtr<$s, $v> {
                new::<$s, $cs, $v, $cv>(capacity, source)
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, cached_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, cached_new_tid_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, cached_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, cached_destroy_tid_f64);
}
