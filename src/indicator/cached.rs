use super::*;
use crate::library::lru_cache::LRUCache as Cache;
use crate::time::*;
use crate::*;
use std::cell::RefCell;

pub struct LRUCache<G, V, I> {
    source: I,
    cache: RefCell<Cache<Time<G>, V>>,
}

impl<G, V, I> LRUCache<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash,
    V: Clone,
    I: Indicator<G, V>,
{
    pub fn new(capacity: usize, source: I) -> Self {
        Self {
            source: source,
            cache: RefCell::new(Cache::new(capacity)),
        }
    }
}

impl<G, V, I> Indicator<G, V> for LRUCache<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy,
    V: Clone,
    I: Indicator<G, V>,
{
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

impl<G, V, I> FuncIndicator<G, V> for LRUCache<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy,
    V: Clone,
    I: FuncIndicator<G, V>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V> {
        let maybe = self.cache.borrow_mut().get(&time).map(|v| v.clone());
        match maybe {
            Some(v) => MaybeValue::Value(v),
            None => match self.source.value(time) {
                MaybeValue::Value(v) => {
                    self.cache.borrow_mut().insert(time, v.clone());
                    MaybeValue::Value(v)
                }
                MaybeValue::OutOfRange => MaybeValue::OutOfRange
            },
        }
    }
}

// #[cfg(ffi)]
mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;
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
                    i_ptr: ptr::null_mut(),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: IPtr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.f_ptr);
                destroy(ptr.i_ptr);
            }
        };
    }

    define_cached_methods!(f64, cached_new_f64, cached_destroy_f64);
}
