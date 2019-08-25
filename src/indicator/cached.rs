use super::*;
use crate::*;
use std::cell::RefCell;

pub struct Cached<T, U> {
    source: T,
    cache: RefCell<VecCache<U>>,
}

impl<T, U> Cached<T, U>
where
    U: Copy,
{
    pub fn new(source: T) -> Self {
        Self {
            source: source,
            cache: RefCell::new(VecCache::new()),
        }
    }
}

impl<T, U> Indicator<U> for Cached<T, U>
where
    T: Indicator<U>,
    U: Copy,
{
    fn value(&self, index: isize) -> Option<U> {
        let maybe_value = { self.cache.borrow().get(index) };
        if let Some(value) = maybe_value {
            Some(value)
        } else {
            let value = self.source.value(index);
            self.cache.borrow_mut().set(index, value);
            value
        }
    }
}

pub struct VecCache<T> {
    cache: Vec<Option<T>>,
    cache_minus: Vec<Option<T>>,
}

impl<T> VecCache<T>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            cache_minus: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: Vec::with_capacity(capacity),
            cache_minus: Vec::new(),
        }
    }

    pub fn get(&self, index: isize) -> Option<T> {
        if index >= 0 {
            let index = index as usize;
            if index < self.cache.len() {
                self.cache[index]
            } else {
                None
            }
        } else {
            let index = -index as usize;
            if index < self.cache_minus.len() {
                self.cache_minus[index]
            } else {
                None
            }
        }
    }

    pub fn set(&mut self, index: isize, value: Option<T>) {
        if index >= 0 {
            let index = index as usize;
            if index < self.cache.len() {
            } else {
                for _ in 0..(index + 1 - self.cache.len()) {
                    self.cache.push(None);
                }
            }
            self.cache[index] = value;
        } else {
            let index = -index as usize;
            if index < self.cache_minus.len() {
            } else {
                for _ in 0..(index + 1 - self.cache_minus.len()) {
                    self.cache_minus.push(None);
                }
            }
            self.cache_minus[index] = value;
        }
    }
}

use std::mem::drop;
use std::os::raw::*;
use std::ptr;
use std::rc::Rc;

macro_rules! define_cached_methods {
    ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $new(
            source: *mut IndicatorPtr<$t>,
        ) -> *mut Rc<RefCell<Cached<IndicatorPtr<$t>, $t>>> {
            let source = (*source).clone();
            let cached = Rc::new(RefCell::new(Cached::new(source)));
            Box::into_raw(Box::new(cached))
        }

        #[no_mangle]
        pub unsafe extern "C" fn $trait(
            obj: *mut Rc<RefCell<Cached<IndicatorPtr<$t>, $t>>>,
        ) -> *mut IndicatorPtr<$t> {
            if obj.is_null() {
                return ptr::null_mut();
            }
            Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
        }

        #[no_mangle]
        pub unsafe extern "C" fn $destroy(
            obj: *mut Rc<RefCell<Cached<IndicatorPtr<$t>, $t>>>,
        ) {
            if obj.is_null() {
                return;
            }
            let boxed = Box::from_raw(obj);
            drop(boxed);
        }
    };
}

define_cached_methods!(f64, cached_new_f64, cached_trait_f64, cached_destroy_f64);
