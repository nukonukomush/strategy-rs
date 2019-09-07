use super::*;
use crate::seq::*;
use crate::transaction::*;
use chrono::prelude::*;

pub struct TransactionIndicator<G, T> {
    source: TransactionHistories<T>,
    granularity: G,
}

impl<G, V, T> Indicator<G, V> for TransactionIndicator<G, T>
where
    G: Granularity + Copy,
    T: Transaction,
{
    fn granularity(&self) -> G {
        self.granularity
    }
}

impl<G, T> FuncIndicator<G, Option<Box<[T]>>> for TransactionIndicator<G, T>
where
    G: Granularity + Copy,
    T: Transaction + Clone,
{
    fn value(&self, time: Time<G>) -> MaybeValue<Option<Box<[T]>>> {
        let start = time.into();
        match self.source.latest_time() {
            Some(t) if start <= t => {
                let end = (time + 1).into();
                MaybeValue::Value(self.source.get_by_time_range(start, end))
            }
            _ => MaybeValue::OutOfRange,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test() {
    //     let histories = TransactionHistories::new();

    // }
}
