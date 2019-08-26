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
    fn value(&self, time: Time<G>) -> Option<V> {
        let maybe = self.cache.borrow_mut().get(&time).map(|v| v.clone());
        match maybe {
            Some(v) => Some(v),
            None => match self.source.value(time) {
                Some(v) => {
                    self.cache.borrow_mut().insert(time, v.clone());
                    Some(v)
                }
                None => None,
            },
        }
    }
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    #[repr(C)]
    pub struct Ptr<V> {
        b_ptr: *mut Rc<RefCell<LRUCache<VarGranularity, V, IndicatorPtr<V>>>>,
        t_ptr: *mut IndicatorPtr<V>,
    }

    macro_rules! define_cached_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                capacity: c_int,
                source: *mut IndicatorPtr<$t>,
            ) -> Ptr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(LRUCache::new(capacity as usize, source)));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    t_ptr: Box::into_raw(Box::new(IndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: Ptr<$t>) {
                destroy(ptr.b_ptr);
                destroy(ptr.t_ptr);
            }
        };
    }

    define_cached_methods!(f64, cached_new_f64, cached_destroy_f64);
}
