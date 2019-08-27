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

pub struct Zip<G, V1, V2, I1, I2> {
    source_1: I1,
    source_2: I2,
    p1: std::marker::PhantomData<G>,
    p2: std::marker::PhantomData<V1>,
    p3: std::marker::PhantomData<V2>,
}

impl<G, V1, V2, I1, I2> Zip<G, V1, V2, I1, I2> {
    pub fn new(source_1: I1, source_2: I2) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            p1: std::marker::PhantomData,
            p2: std::marker::PhantomData,
            p3: std::marker::PhantomData,
        }
    }
}

impl<G, V1, V2, I1, I2> Indicator<G, (V1, V2)> for Zip<G, V1, V2, I1, I2>
where
    I1: Indicator<G, V1>,
    I2: Indicator<G, V2>,
{
    fn granularity(&self) -> G {
        self.source_1.granularity()
    }
}

impl<G, V1, V2, I1, I2> FuncIndicator<G, (V1, V2)> for Zip<G, V1, V2, I1, I2>
where
    G: Granularity + Copy,
    I1: FuncIndicator<G, V1>,
    I2: FuncIndicator<G, V2>,
{
    fn value(&self, time: Time<G>) -> MaybeValue<(V1, V2)> {
        let v1 = try_value!(self.source_1.value(time));
        let v2 = try_value!(self.source_2.value(time));
        MaybeValue::Value((v1, v2))
    }
}

impl<G, V1, V2, I1, I2> IterIndicator<G, (V1, V2)> for Zip<G, V1, V2, I1, I2>
where
    I1: IterIndicator<G, V1>,
    I2: IterIndicator<G, V2>,
{
    fn next(&mut self) -> MaybeValue<(V1, V2)> {
        let v1 = try_value!(self.source_1.next());
        let v2 = try_value!(self.source_2.next());
        MaybeValue::Value((v1, v2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;
    use MaybeValue::*;

    #[test]
    fn test_zip() {
        let offset = Time::new(0, S5);
        let source_1 = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let source_2 = vec![0, -1, 0, 1, 0_i32];
        let expect = vec![Value(0.0), Value(2.0), Value(0.0), Value(4.0), Value(0.0)];
        let vec_1 = VecIndicator::new(offset, source_1);
        let vec_2 = VecIndicator::new(offset, source_2);
        let func = vec_1.zip(vec_2).map(|(v1, v2)| v1 * v2.abs() as f64);

        let result = (0..5).map(|i| func.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
