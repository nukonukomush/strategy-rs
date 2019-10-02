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

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct TransactionId(pub i64);

impl Add<i64> for TransactionId {
    type Output = TransactionId;
    fn add(self, other: i64) -> Self::Output {
        TransactionId(self.0 + other)
    }
}

impl Sub<i64> for TransactionId {
    type Output = TransactionId;
    fn sub(self, other: i64) -> Self::Output {
        TransactionId(self.0 - other)
    }
}

impl Sequence for TransactionId {
    fn distance_from(&self, offset: &Self) -> i64 {
        self.0 - offset.0
    }
}

impl Into<i64> for TransactionId {
    fn into(self) -> i64 {
        self.0
    }
}

impl From<i64> for TransactionId {
    fn from(i: i64) -> Self {
        TransactionId(i)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct TickId(pub i64);

impl Add<i64> for TickId {
    type Output = TickId;
    fn add(self, other: i64) -> Self::Output {
        TickId(self.0 + other)
    }
}

impl Sub<i64> for TickId {
    type Output = TickId;
    fn sub(self, other: i64) -> Self::Output {
        TickId(self.0 - other)
    }
}

impl Sequence for TickId {
    fn distance_from(&self, offset: &Self) -> i64 {
        self.0 - offset.0
    }
}

impl Into<i64> for TickId {
    fn into(self) -> i64 {
        self.0
    }
}

impl From<i64> for TickId {
    fn from(i: i64) -> Self {
        TickId(i)
    }
}
