use super::*;
use crate::library::lru_cache::LRUCache;
use crate::seq::*;
use std::cell::RefCell;

pub struct ComplementWithLastValue<S, V, I> {
    source: I,
    cache: RefCell<LRUCache<S, Option<V>>>,
    phantom: std::marker::PhantomData<S>,
}

impl<S, V, I> ComplementWithLastValue<S, V, I>
where
    S: Sequence,
    V: Clone,
{
    pub fn new(source: I, capacity: usize) -> Self {
        Self {
            source: source,
            cache: RefCell::new(LRUCache::new(capacity)),
            phantom: std::marker::PhantomData,
        }
    }

    fn get_cache(&self, seq: S) -> Option<Option<V>> {
        self.cache.borrow_mut().get(&seq).map(|v| v.clone())
    }

    fn set_cache(&self, seq: S, value: Option<V>) {
        self.cache.borrow_mut().insert(seq, value);
    }
}

impl<S, V, I> Indicator<S, V> for ComplementWithLastValue<S, V, I>
where
    S: Sequence,
    V: Clone,
    I: Indicator<S, Option<V>>,
{
}

impl<S, V, I> FuncIndicator<S, V> for ComplementWithLastValue<S, V, I>
where
    S: Sequence,
    V: Clone,
    I: FuncIndicator<S, Option<V>>,
{
    fn value(&self, seq: S) -> MaybeValue<V> {
        let cache = self.get_cache(seq);
        match cache {
            Some(Some(v)) => MaybeValue::Value(v),
            Some(None) => MaybeValue::OutOfRange,
            None => {
                let src_value = try_value!(self.source.value(seq));
                let value = match src_value {
                    Some(v) => MaybeValue::Value(v),
                    None => self.value(seq - 1),
                };
                match value.clone() {
                    MaybeValue::Value(v) => self.set_cache(seq, Some(v)),
                    MaybeValue::OutOfRange => self.set_cache(seq, None),
                };
                value
            }
        }
    }
}

// #[cfg(ffi)]
pub mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, ComplementWithLastValue<S, V, FuncIndicatorPtr<S, Option<V>>>>;

    pub unsafe fn new<S, CS, V, CV>(
        source: *mut FuncIndicatorPtr<S, Option<V>>,
        capacity: c_int,
    ) -> IPtr<S, V>
    where
        S: Sequence + 'static,
        CS: Into<S>,
        V: Clone + 'static,
        CV: Into<V>,
    {
        let source = (*source).clone();
        let ptr = Rc::new(RefCell::new(ComplementWithLastValue::new(
            source,
            capacity as usize,
        )));
        Ptr {
            b_ptr: Box::into_raw(Box::new(ptr.clone())),
            f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
        }
    }

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                source: *mut FuncIndicatorPtr<$s, Option<$v>>,
                capacity: c_int,
            ) -> IPtr<$s, $v> {
                let source = (*source).clone();
                let ptr = Rc::new(RefCell::new(ComplementWithLastValue::new(
                    source,
                    capacity as usize,
                )));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, cmpl_new_time_f64);
    define_new!(TransactionId, i64, f64, f64, cmpl_new_tid_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, cmpl_destroy_time_f64);
    define_destroy!(IPtr<TransactionId, f64>, cmpl_destroy_tid_f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::granularity::*;
    use crate::storage::*;
    use MaybeValue::*;

    #[test]
    fn test_cmpl() {
        let offset = Time::<S5>::new(0);
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
