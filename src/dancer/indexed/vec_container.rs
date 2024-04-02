use super::*;

impl<T: Clone> AbstractContainer for Vec<T> {
    type Item = T;

    fn new(capacity: usize) -> Self {
        Vec::<T>::with_capacity(capacity)
    }

    fn push(&mut self, value: Self::Item) {
        self.push(value)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn resize(&mut self, new_len: usize, value: Self::Item) {
        self.resize(new_len, value)
    }
}

pub struct VecContainerType {}
impl AbstractContainerType for VecContainerType {
    type Container<T: Clone> = Vec<T>;
}
