use super::*;
use crate::seq::*;
use crate::library::lru_cache::LRUCache as Cache;
use crate::time::*;
use crate::*;
use std::cell::RefCell;

pub struct LRUCache<S, V, I> {
    source: I,
    cache: RefCell<Cache<S, V>>,
}

impl<S, V, I> LRUCache<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash,
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
    // S: Granularity + Eq + std::hash::Hash + Copy,
    S: Sequence,
    V: Clone,
    I: Indicator<S, V>,
{
    // fn granularity(&self) -> S {
    //     self.source.granularity()
    // }
}

impl<S, V, I> FuncIndicator<S, V> for LRUCache<S, V, I>
where
    // S: Granularity + Eq + std::hash::Hash + Copy,
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

#[cfg(ffi)]
mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::seq::ffi::*;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    type IPtr<V> = Ptr<V, LRUCache<VarGranularity, V, FuncIndicatorPtr<V>>>;

    macro_rules! define_cached_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                capacity: c_int,
                source: *mut FuncIndicatorPtr<$t>,
            ) -> IPtr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(LRUCache::new(capacity as usize, source)));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: IPtr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.f_ptr);
            }
        };
    }

    define_cached_methods!(f64, cached_new_f64, cached_destroy_f64);
}
