use super::*;
use crate::time::*;

pub struct ConvertGranularity<I> {
    source: I,
}

// impl<G1, G2, V, I> Indicator<G2, V> for ConvertGranularity<I>
// where
//     G1: Granularity,
//     G2: Granularity,
//     I: Indicator<G1, V>,
// {
//     fn value(&self, time: Time<G2>) -> Option<V> {
//         self.
//     }
// }
