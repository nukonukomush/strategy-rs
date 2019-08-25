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
    G: Granularity,
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
    G: Granularity,
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
}

// use std::mem::drop;
// use std::os::raw::*;
// use std::ptr;
// use std::rc::Rc;

// macro_rules! define_cached_methods {
//     ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
//         #[no_mangle]
//         pub unsafe extern "C" fn $new(
//             source: *mut IndicatorPtr<$t>,
//         ) -> *mut Rc<RefCell<Cached<IndicatorPtr<$t>, $t>>> {
//             let source = (*source).clone();
//             let cached = Rc::new(RefCell::new(Cached::new(source)));
//             Box::into_raw(Box::new(cached))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $trait(
//             obj: *mut Rc<RefCell<Cached<IndicatorPtr<$t>, $t>>>,
//         ) -> *mut IndicatorPtr<$t> {
//             if obj.is_null() {
//                 return ptr::null_mut();
//             }
//             Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn $destroy(
//             obj: *mut Rc<RefCell<Cached<IndicatorPtr<$t>, $t>>>,
//         ) {
//             if obj.is_null() {
//                 return;
//             }
//             let boxed = Box::from_raw(obj);
//             drop(boxed);
//         }
//     };
// }

// define_cached_methods!(f64, cached_new_f64, cached_trait_f64, cached_destroy_f64);
