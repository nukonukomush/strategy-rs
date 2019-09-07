use std::ops::Add;
use std::ops::Sub;

pub trait Sequence:
    Ord
    + Eq
    + Copy
    + Add<i64, Output = Self>
    + Sub<i64, Output = Self>
    + std::hash::Hash
    + std::fmt::Debug
{
    fn distance_from(&self, offset: &Self) -> i64;
    fn range_to_end(&self, end: Self) -> SeqRangeTo<Self>
    where
        Self: Sized,
    {
        SeqRangeTo {
            current: *self,
            end: end,
        }
    }
}

pub struct SeqRangeTo<S> {
    current: S,
    end: S,
}

impl<S> std::iter::Iterator for SeqRangeTo<S>
where
    S: Sequence,
{
    type Item = S;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current + 1;
        if next >= self.end {
            None
        } else {
            self.current = next;
            Some(next)
        }
    }
}
