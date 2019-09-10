pub mod library;

pub mod ffi {
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
    }

    impl<T> Default for COption<T>
    where
        T: Default,
    {
        fn default() -> Self {
            Self::none()
        }
    }

    impl<T> Into<Option<T>> for COption<T>
    {
        fn into(self) -> Option<T> {
            if self.is_some == 0 {
                None
            } else {
                Some(self.value)
            }
        }
    }

    impl<T> From<Option<T>> for COption<T>
    where
        T: Default,
    {
        fn from(value: Option<T>) -> Self {
            match value {
                Some(value) => Self::some(value),
                None => Self::none(),
            }
        }
    }
}

pub mod transaction;
pub mod indicator;
pub mod position;
pub mod seq;
pub mod granularity;
pub mod time;
// pub mod strategy;
pub mod ticket;

use indicator::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
