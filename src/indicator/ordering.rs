use super::*;
use crate::*;

pub struct Ordering<I1, I2, V> {
    source_1: I1,
    source_2: I2,
    phantom: std::marker::PhantomData<V>,
}

impl<I1, I2, V> Ordering<I1, I2, V>
where
    I1: Indicator<V>,
    I2: Indicator<V>,
    V: PartialOrd,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<I1, I2, V> Indicator<std::cmp::Ordering> for Ordering<I1, I2, V>
where
    I1: Indicator<V>,
    I2: Indicator<V>,
    V: PartialOrd,
{
    fn value(&self, index: isize) -> Option<std::cmp::Ordering> {
        if let (Some(val_1), Some(val_2)) = (self.source_1.value(index), self.source_2.value(index))
        {
            // TODO: don't use unwrap
            let ord = val_1.partial_cmp(&val_2).unwrap();
            Some(ord)
        } else {
            None
        }
    }
}

pub mod value {
    #[repr(C)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub enum Ordering {
        Less = -1,
        Equal = 0,
        Greater = 1,
    }

    impl Ordering {
        pub fn from_std(src: std::cmp::Ordering) -> Self {
            match src {
                std::cmp::Ordering::Equal => Ordering::Equal,
                std::cmp::Ordering::Greater => Ordering::Greater,
                std::cmp::Ordering::Less => Ordering::Less,
            }
        }
    }
}

impl<I1, I2, V> Indicator<value::Ordering> for Ordering<I1, I2, V>
where
    I1: Indicator<V>,
    I2: Indicator<V>,
    V: PartialOrd,
{
    fn value(&self, index: isize) -> Option<value::Ordering> {
        if let (Some(val_1), Some(val_2)) = (self.source_1.value(index), self.source_2.value(index))
        {
            // TODO: don't use unwrap
            let ord = val_1.partial_cmp(&val_2).unwrap();
            Some(value::Ordering::from_std(ord))
        } else {
            None
        }
    }
}

use std::cell::RefCell;
use std::mem::drop;
use std::os::raw::*;
use std::ptr;
use std::rc::Rc;

macro_rules! define_ordering_methods {
    ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $new(
            source_1: *mut IndicatorPtr<$t>,
            source_2: *mut IndicatorPtr<$t>,
        ) -> *mut Rc<RefCell<Ordering<IndicatorPtr<$t>, IndicatorPtr<$t>, $t>>> {
            let source_1 = (*source_1).clone();
            let source_2 = (*source_2).clone();
            let ordering = Rc::new(RefCell::new(Ordering::new(source_1, source_2)));
            Box::into_raw(Box::new(ordering))
        }

        #[no_mangle]
        pub unsafe extern "C" fn $trait(
            obj: *mut Rc<RefCell<Ordering<IndicatorPtr<$t>, IndicatorPtr<$t>, $t>>>,
        ) -> *mut IndicatorPtr<value::Ordering> {
            if obj.is_null() {
                return ptr::null_mut();
            }
            Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
        }

        #[no_mangle]
        pub unsafe extern "C" fn $destroy(
            obj: *mut Rc<RefCell<Ordering<IndicatorPtr<$t>, IndicatorPtr<$t>, $t>>>,
        ) {
            if obj.is_null() {
                return;
            }
            let boxed = Box::from_raw(obj);
            drop(boxed);
        }
    };
}

define_ordering_methods!(
    f64,
    ordering_new_f64,
    ordering_trait_f64,
    ordering_destroy_f64
);
