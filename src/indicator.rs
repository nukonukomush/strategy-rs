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
