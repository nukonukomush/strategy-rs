use super::*;
use crate::*;

pub struct Ordering<G, V, I1, I2> {
    source_1: I1,
    source_2: I2,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V>,
}

impl<G, V, I1, I2> Ordering<G, V, I1, I2>
where
    G: Granularity,
    V: PartialOrd,
    I1: Indicator<G, V>,
    I2: Indicator<G, V>,
{
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
        }
    }
}

// impl<G, V, I1, I2> Indicator<std::cmp::Ordering> for Ordering<G, V, I1, I2>
// where
//     I1: Indicator<V>,
//     I2: Indicator<V>,
//     V: PartialOrd,
// {
//     fn value(&self, index: isize) -> Option<std::cmp::Ordering> {
//         if let (Some(val_1), Some(val_2)) = (self.source_1.value(index), self.source_2.value(index))
//         {
//             // TODO: don't use unwrap
//             let ord = val_1.partial_cmp(&val_2).unwrap();
//             Some(ord)
//         } else {
//             None
//         }
//     }
// }

// #[repr(C)]
// #[derive(PartialEq, Eq, Debug, Clone, Copy)]
// pub enum OrderingValue {
//     Less = -1,
//     Equal = 0,
//     Greater = 1,
// }

// impl OrderingValue {
//     pub fn from_std(src: std::cmp::Ordering) -> Self {
//         match src {
//             std::cmp::Ordering::Equal => OrderingValue::Equal,
//             std::cmp::Ordering::Greater => OrderingValue::Greater,
//             std::cmp::Ordering::Less => OrderingValue::Less,
//         }
//     }
// }

impl<G, V, I1, I2> Indicator<G, std::cmp::Ordering> for Ordering<G, V, I1, I2>
where
    G: Granularity + Copy,
    V: PartialOrd,
    I1: Indicator<G, V>,
    I2: Indicator<G, V>,
{
    fn value(&self, time: Time<G>) -> Option<std::cmp::Ordering> {
        if let (Some(val_1), Some(val_2)) = (self.source_1.value(time), self.source_2.value(time)) {
            // TODO: don't use unwrap
            let ord = val_1.partial_cmp(&val_2).unwrap();
            Some(ord)
        } else {
            None
        }
    }
    fn granularity(&self) -> G {
        self.source_1.granularity()
    }
}
