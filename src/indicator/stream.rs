use super::*;
use crate::time::*;

pub struct Map<G, V1, V2, I, F> {
    source: I,
    func: F,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V1>,
    p3: std::marker::PhantomData<V2>,
}

impl<G, V1, V2, I, F> Map<G, V1, V2, I, F> {
    pub fn new(source: I, func: F) -> Self {
        Self {
            source: source,
            func: func,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
            p3: std::marker::PhantomData,
        }
    }
}

impl<G, V1, V2, I, F> Indicator<G, V2> for Map<G, V1, V2, I, F>
where
    I: Indicator<G, V1>,
    F: FnMut(V1) -> V2,
{
    fn granularity(&self) -> G {
        self.source.granularity()
    }
}

impl<G, V1, V2, I, F> FuncIndicator<G, V2> for Map<G, V1, V2, I, F>
where
    I: FuncIndicator<G, V1>,
    F: Fn(V1) -> V2,
{
    fn value(&self, time: Time<G>) -> MaybeValue<V2> {
        self.source.value(time).map(|v| (self.func)(v))
    }
}

impl<G, V1, V2, I, F> IterIndicator<G, V2> for Map<G, V1, V2, I, F>
where
    I: IterIndicator<G, V1>,
    F: FnMut(V1) -> V2,
{
    fn next(&mut self) -> MaybeValue<V2> {
        self.source.next().map(|v| (self.func)(v))
    }
}

// TODO: then は使わないかも？けす
// pub struct Then<G, V1, V2, I, F> {
//     source: I,
//     func: F,
//     p1: std::marker::PhantomData<G>,
//     p2: std::marker::PhantomData<V1>,
//     p3: std::marker::PhantomData<V2>,
// }

// impl<G, V1, V2, I, F> Then<G, V1, V2, I, F> {
//     pub fn new(source: I, func: F) -> Self {
//         Self {
//             source: source,
//             func: func,
//             p1: std::marker::PhantomData,
//             p2: std::marker::PhantomData,
//             p3: std::marker::PhantomData,
//         }
//     }
// }

// impl<G, V1, V2, I, F> Indicator<G, V2> for Then<G, V1, V2, I, F>
// where
//     I: Indicator<G, V1>,
//     F: Fn(MaybeValue<V1>) -> MaybeValue<V2>,
// {
//     fn granularity(&self) -> G {
//         self.source.granularity()
//     }
// }

// impl<G, V1, V2, I, F> FuncIndicator<G, V2> for Then<G, V1, V2, I, F>
// where
//     I: FuncIndicator<G, V1>,
//     F: Fn(MaybeValue<V1>) -> MaybeValue<V2>,
// {
//     fn value(&self, time: Time<G>) -> MaybeValue<V2> {
//         (self.func)(self.source.value(time))
//     }
// }
