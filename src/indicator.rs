pub trait Indicator<T> {
    fn value(&self, index: isize) -> Option<T>;
}

impl<T> Indicator<T> for Vec<T>
where
    T: Clone,
{
    fn value(&self, index: isize) -> Option<T> {
        if index >= 0 {
            let index = index as usize;
            if self.len() > index {
                Some(self[index].clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expect = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];

        let result = (0..5).map(|i| source.value(i)).collect::<Vec<_>>();
        assert_eq!(result, expect);
    }
}
