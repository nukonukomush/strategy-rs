use super::*;
use crate::library::lru_cache::LRUCache;
use std::cell::RefCell;

pub struct ComplementWithLastValue<G, V, I> {
    source: I,
    cache: RefCell<LRUCache<Time<G>, Option<V>>>,
    phantom: std::marker::PhantomData<G>,
}

impl<G, V, I> ComplementWithLastValue<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy,
    V: Clone,
    I: FuncIndicator<G, Option<V>>,
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
            phantom: std::marker::PhantomData,
        }
    }

    fn get_from_cache(&self, time: Time<G>) -> MaybeValue<Option<V>> {
        let maybe = self.cache.borrow_mut().get(&time).map(|v| v.clone());
        match maybe {
            Some(v) => MaybeValue::Value(v),
            None => match self.source.value(time) {
                MaybeValue::Value(v) => {
                    self.cache.borrow_mut().insert(time, v.clone());
                    MaybeValue::Value(v)
                }
                MaybeValue::OutOfRange => MaybeValue::OutOfRange,
            },
        }
    }
}

impl<G, V, I> Indicator<G, V> for ComplementWithLastValue<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy + Debug,
    V: Clone + Debug,
    I: Indicator<G, Option<V>>,
{
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

use std::fmt::Debug;
impl<G, V, I> FuncIndicator<G, V> for ComplementWithLastValue<G, V, I>
where
    G: Granularity + Eq + std::hash::Hash + Copy + Debug,
    V: Clone + Debug,
    I: FuncIndicator<G, Option<V>>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V> {
        let mut t = time;
        let mut value = try_value!(self.get_from_cache(t));
        while value.is_none() {
            t = t - 1;
            value = try_value!(self.get_from_cache(t));
        }
        self.cache.borrow_mut().insert(time, value.clone());
        match value {
            Some(v) => MaybeValue::Value(v),
            None => MaybeValue::OutOfRange,
        }
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
    use crate::storage::*;
    use MaybeValue::*;

    #[test]
    fn test_cmpl() {
        let offset = Time::new(0, S5);
        let mut storage = Storage::new(offset);
        storage.add(offset + 2, 1.0);
        storage.add(offset + 3, 2.0);
        storage.add(offset + 5, 3.0);
        storage.add(offset + 8, 4.0);
        let expect = vec![
            OutOfRange,
            OutOfRange,
            Value(1.0),
            Value(2.0),
            Value(2.0),
            Value(3.0),
            Value(3.0),
            Value(3.0),
            Value(4.0),
            OutOfRange,
        ];

        let cmpl = ComplementWithLastValue::new(storage, 10);
        let result = (0..10).map(|i| cmpl.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
