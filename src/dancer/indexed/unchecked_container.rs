use std::mem::{align_of, size_of};

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::null_mut;

use super::*;

pub struct Unchecked<T: Clone> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
}

impl<T: Clone> Unchecked<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let size = size_of::<T>() * capacity;
        let align = align_of::<T>();
        debug_assert!(size > 0);
        let ptr = unsafe { alloc(Layout::from_size_align_unchecked(size, align)) as *mut T };
        Self {
            ptr,
            capacity,
            len: 0,
        }
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
    fn len(&self) -> usize {
        self.len
    }

    fn push(&mut self, value: T) {
        debug_assert!(self.len < self.capacity);
        unsafe {
            self.ptr.offset(self.len as isize).write(value);
        }
        self.len += 1;
    }

    fn resize(&mut self, new_len:usize, value: T) {
        debug_assert!(new_len>=self.len);
        debug_assert!(new_len<=self.capacity);
        for i in self.len..new_len {
            unsafe {
                self.ptr.offset(i as isize).write(value.clone());
            }
        }
        self.len=new_len;
    }
}

impl<T:Clone> Drop for Unchecked<T> {
    fn drop(&mut self) {
        let size=size_of::<T>()*self.len;
        let align=align_of::<T>();
        unsafe {
            dealloc(self.ptr as *mut u8, Layout::from_size_align_unchecked(size, align));
        }
    }
}

impl<T: Clone> Index<usize> for Unchecked<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len);
        unsafe { &*self.ptr.offset(index as isize) }
    }
}

impl<T: Clone> IndexMut<usize> for Unchecked<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len);
        unsafe { &mut *self.ptr.offset(index as isize) }
    }
}

impl<T: Clone> AbstractContainer for Unchecked<T> {
    type Item = T;

    fn new(capacity: usize) -> Self {
        Unchecked::<T>::with_capacity(capacity)
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

pub struct UncheckedContainerType {}
impl AbstractContainerType for UncheckedContainerType {
    type Container<T: Clone> = Unchecked<T>;
}
