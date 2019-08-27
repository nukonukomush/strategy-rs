use super::*;
use crate::library::lru_cache::LRUCache;
// use crate::indicator::cached::LRUCache;
use crate::indicator::stream::Then;
use crate::time::*;
use std::cell::RefCell;

pub struct ComplementWithLastValue<G, V, I> {
    source: I,
    max_loop: usize,
    cache: RefCell<LRUCache<Time<G>, Option<V>>>,
    // cache: Option<
    //     LRUCache<G, Option<V>, Then<G, V, Option<V>, &'a dyn Indicator<G, V>, fn(Option<V>) -> Option<Option<V>>>>,
    // >,
    phantom: std::marker::PhantomData<G>,
}

// fn wrap_some<V>(v: value) -> Option<V> {
//     So
// }

impl<G, V, I> ComplementWithLastValue<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy,
    V: Clone,
    I: Indicator<G, V>,
{
    pub fn new(source: I, max_loop: usize, capacity: usize) -> Self {
        // let mut this = Self {
        let this = Self {
            source: source,
            max_loop: max_loop,
            // cache: None,
            cache: RefCell::new(LRUCache::new(capacity)),
            phantom: std::marker::PhantomData,
        };
        this
        // this.cache = Some(LRUCache::new(capacity, Then::new(&this, |v| Some(v))));
        // this
    }

    // fn get(&self, time: Time<G>) -> Option<Option<V>> {
    //     let maybe = self.cache.borrow_mut().get(&time).map(|v| v.clone());
    //     match maybe {
    //         Some(v) => Some(v),
    //         None => match self.source.value(time) {},
    //     }
    // }

    // fn search_source(&self, time: Time<G>) -> Option<V> {
    //     let mut loop_count = 0;
    //     let mut time = time;
    //     let mut maybe = self.source.value(time);
    //     while maybe.is_none() && loop_count < self.max_loop {
    //         loop_count += 1;
    //         time = time - 1;
    //         maybe = self.source.value(time);
    //     }
    //     maybe
    // }

    fn get_from_cache(&self, time: Time<G>) -> Option<Option<V>> {
        let maybe = self.cache.borrow_mut().get(&time).map(|v| v.clone());
        match maybe {
            Some(v) => Some(v),
            None => match self.source.value(time) {
                Some(v) => {
                    self.cache.borrow_mut().insert(time, Some(v.clone()));
                    Some(Some(v))
                }
                None => None,
            },
        }
    }
}

use std::fmt::Debug;
impl<G, V, I> Indicator<G, V> for ComplementWithLastValue<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy + Debug,
    V: Clone + Debug,
    I: Indicator<G, V>,
{
    fn value(&self, time: Time<G>) -> Option<V> {
        // // self.search_source(time)
        // None
        let mut loop_count = 0;
        let mut t = time;
        let mut maybe = self.get_from_cache(t);
        while maybe.is_none() && loop_count < self.max_loop {
            loop_count += 1;
            t = t - 1;
            maybe = self.get_from_cache(t);
        }
        let ret = match maybe {
            Some(v) => {
                self.cache.borrow_mut().insert(time, v.clone());
                v
            },
            None => {
                self.cache.borrow_mut().insert(time, None);
                None
            }
        }         ;
        // println!("{:?}", self.cache);
        ret
    }
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

#[cfg(ffi)]
pub mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::time::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    #[repr(C)]
    pub struct Ptr<V> {
        b_ptr: *mut Rc<RefCell<ComplementWithLastValue<VarGranularity, V, IndicatorPtr<V>>>>,
        t_ptr: *mut IndicatorPtr<V>,
    }

    macro_rules! define_cmpl_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                source: *mut IndicatorPtr<$t>,
                max_loop: c_int,
                capacity: c_int,
            ) -> Ptr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(ComplementWithLastValue::new(
                    source,
                    max_loop as usize,
                    capacity as usize,
                )));
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

    define_cmpl_methods!(f64, cmpl_new_f64, cmpl_destroy_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use MaybeValue::*;
    use crate::vec::*;

    #[test]
    fn test_cmpl() {
        let offset = Time::new(0, S5);
        let mut hash = HashMapIndicator::new(S5);
        hash.insert(offset + 3, 1.0);
        hash.insert(offset + 4, 2.0);
        hash.insert(offset + 6, 3.0);
        let expect = vec![
            None,
            None,
            None,
            Some(1.0),
            Some(2.0),
            Some(2.0),
            Some(3.0),
            Some(3.0),
            Some(3.0),
            Some(3.0),
        ];

        let cmpl = ComplementWithLastValue::new(hash, 10, 10);
        let result = (0..10).map(|i| cmpl.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

    #[test]
    fn test_cmpl_cache() {
        let offset = Time::new(0, S5);
        let mut hash = HashMapIndicator::new(S5);
        hash.insert(offset + 1, 1.0);
        let expect = vec![
            None,
            Some(1.0),
            Some(1.0),
            Some(1.0),
            Some(1.0),
        ];

        let cmpl = ComplementWithLastValue::new(hash, 1, 10);
        let result = (0..5).map(|i| cmpl.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
