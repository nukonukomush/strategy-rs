pub mod indicator;

#[no_mangle]
pub extern "C" fn test_fn() -> i32 {
    1234
}

use std::os::raw::c_char;
#[repr(C)]
pub struct COption<T> {
    is_some: c_char,
    value: T,
}

impl<T> COption<T>
where
    T: Default,
{
    pub fn none() -> Self {
        Self {
            is_some: 0,
            value: Default::default(),
        }
    }

    pub fn some(value: T) -> Self {
        Self {
            is_some: 1,
            value: value,
        }
    }

    pub fn from_option(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::some(value),
            None => Self::none(),
        }
    }
}

use indicator::*;
use std::mem::drop;
use std::os::raw::{c_double, c_int};
use std::ptr;
#[no_mangle]
pub unsafe extern "C" fn vec_new_f64(array: *const c_double, length: c_int) -> *mut Vec<f64> {
    let array: &[c_double] = std::slice::from_raw_parts(array, length as usize);
    let obj = Box::new(array.to_vec());
    Box::into_raw(obj)
}

#[no_mangle]
pub unsafe extern "C" fn vec_destroy_f64(obj: *mut Vec<f64>) {
    if obj.is_null() {
        return;
    }
    let boxed = Box::from_raw(obj);
    drop(boxed);
}

#[no_mangle]
pub unsafe extern "C" fn vec_value_f64(vec: *mut Vec<f64>, i: c_int) -> COption<c_double> {
    print!("{}", vec as isize);
    if vec.is_null() {
        return COption::none();
    }

    let vec = &*vec;
    COption::from_option(vec.value(i as isize))
}

#[cfg(test)]
mod tests {
    use super::*;
    // #[test]
    // fn test_vec() {
    //     let ptr = vec_new_f64();
    //     let v = unsafe { vec_value_f64(ptr, 0) };
    //     unsafe { vec_destroy_f64(ptr) };
    //     assert_eq!(v, 1.0);
    // }
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
