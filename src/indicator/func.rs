use super::*;
use crate::time::*;

// TODO: Box じゃなくする
pub struct Func2<G, VIn1, VIn2, VOut, I1, I2> {
    source_1: I1,
    source_2: I2,
    // closure: Box<dyn Fn(VIn1, VIn2) -> VOut>,
    closure: Box<dyn Fn(VIn1, VIn2) -> VOut>,
    phantom: std::marker::PhantomData<G>,
}

impl<G, VIn1, VIn2, VOut, I1, I2> Func2<G, VIn1, VIn2, VOut, I1, I2>
where
    I1: Indicator<G, VIn1>,
    I2: Indicator<G, VIn2>,
{
    pub fn new(source_1: I1, source_2: I2, closure: Box<dyn Fn(VIn1, VIn2) -> VOut>) -> Self {
        Self {
            source_1: source_1,
            source_2: source_2,
            closure: closure,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, VIn1, VIn2, VOut, I1, I2> Indicator<G, VOut> for Func2<G, VIn1, VIn2, VOut, I1, I2>
where
    G: Granularity + Copy,
    I1: Indicator<G, VIn1>,
    I2: Indicator<G, VIn2>,
{
    fn value(&self, time: Time<G>) -> Option<VOut> {
        let in1 = self.source_1.value(time)?;
        let in2 = self.source_2.value(time)?;
        let out = (self.closure)(in1, in2);
        Some(out)
    }
    fn granularity(&self) -> G {
        self.source_1.granularity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::*;

    #[test]
    fn test_func2() {
        let offset = Time::new(0, S5);
        let source_1 = vec![1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let source_2 = vec![0, -1, 0, 1, 0_i32];
        let expect = vec![Some(0.0), Some(2.0), Some(0.0), Some(4.0), Some(0.0)];
        let vec_1 = VecIndicator::new(offset, source_1);
        let vec_2 = VecIndicator::new(offset, source_2);
        let func = Func2::new(vec_1, vec_2, Box::new(|v1, v2: i32| v1 * v2.abs() as f64));

        let result = (0..5).map(|i| func.value(offset + i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }

}
