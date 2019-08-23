use crate::*;

impl<T> Indicator<T> for Vec<T>
where
    T: Clone,
{
    fn value(&self, index: isize) -> Option<T> {
        if index >= 0 {
            let index = index as usize;
            if self.len() > index {
                Some(self[index].clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

use std::mem::drop;
use std::os::raw::*;
use std::ptr;

use std::cell::RefCell;
use std::rc::Rc;

macro_rules! define_vec_methods {
    ($t:ty, $new:ident, $trait:ident, $destroy:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $new(
            array: *const $t,
            length: c_int,
        ) -> *mut Rc<RefCell<Vec<$t>>> {
            let array: &[$t] = std::slice::from_raw_parts(array, length as usize);
            let obj = Box::new(Rc::new(RefCell::new(array.to_vec())));
            Box::into_raw(obj)
        }

        #[no_mangle]
        pub unsafe extern "C" fn $trait(
            obj: *mut Rc<RefCell<Vec<$t>>>,
        ) -> *mut IndicatorPtr<$t> {
            if obj.is_null() {
                return ptr::null_mut();
            }
            Box::into_raw(Box::new(IndicatorPtr((*obj).clone())))
        }

        #[no_mangle]
        pub unsafe extern "C" fn $destroy(obj: *mut Rc<RefCell<Vec<$t>>>) {
            if obj.is_null() {
                return;
            }
            // ここ Box にする必要ある？？
            let boxed = Box::from_raw(obj);
            drop(boxed);
        }
    };
}

define_vec_methods!(f64, vec_new_f64, vec_trait_f64, vec_destroy_f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];

        let result = (0..5).map(|i| source.value(i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
