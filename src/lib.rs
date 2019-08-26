pub mod library;

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

// use std::hash::Hash;
// pub trait Time: Eq + Ord + Hash {
// }

pub mod time;
pub mod position;
pub mod indicator;

use indicator::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
