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
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
            phantom: std::marker::PhantomData,
        }
    }

    fn get_cache(&self, time: Time<G>) -> Option<Option<V>> {
        self.cache.borrow_mut().get(&time).map(|v| v.clone())
    }

    fn set_cache(&self, time: Time<G>, value: Option<V>) {
        self.cache.borrow_mut().insert(time, value);
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
        let cache = self.get_cache(time);
        match cache {
            Some(Some(v)) => MaybeValue::Value(v),
            Some(None) => MaybeValue::OutOfRange,
            None => {
                let src_value = try_value!(self.source.value(time));
                let value = match src_value {
                    Some(v) => MaybeValue::Value(v),
                    None => self.value(time - 1),
                };
                match value.clone() {
                    MaybeValue::Value(v) => self.set_cache(time, Some(v)),
                    MaybeValue::OutOfRange => self.set_cache(time, None),
                };
                value
            }
        }
    }
}

// #[cfg(ffi)]
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

    type IPtr<V> = Ptr<V, ComplementWithLastValue<VarGranularity, V, FuncIndicatorPtr<Option<V>>>>;

    macro_rules! define_cmpl_methods {
        ($t:ty, $new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                source: *mut FuncIndicatorPtr<Option<$t>>,
                capacity: c_int,
            ) -> IPtr<$t> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(ComplementWithLastValue::new(
                    source,
                    capacity as usize,
                )));
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
