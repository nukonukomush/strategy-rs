pub trait Indicator<T> {
    fn value(&self, index: isize) -> Option<T>;
}

pub mod vec;
pub mod sma;

