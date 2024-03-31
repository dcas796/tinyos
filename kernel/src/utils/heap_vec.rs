use crate::utils::heap_array::HeapArray;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{ptr, slice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapVecError {
    HeapArrayError(crate::utils::heap_array::HeapArrayError),
}

impl From<crate::utils::heap_array::HeapArrayError> for HeapVecError {
    fn from(value: crate::utils::heap_array::HeapArrayError) -> Self {
        Self::HeapArrayError(value)
    }
}

#[derive(Clone)]
pub struct HeapVec<T> {
    array: HeapArray<T>,
    len: usize,
}

impl<T> HeapVec<T> {
    pub fn new(cap: usize) -> Result<Self, HeapVecError> {
        Ok(Self {
            array: HeapArray::new(cap)?,
            len: 0,
        })
    }

    pub fn new_with_ptr(ptr: NonNull<T>, cap: usize) -> Self {
        Self {
            array: HeapArray::new_with_ptr(ptr, cap),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cap(&self) -> usize {
        self.array.len()
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(self.len + 1 <= self.cap(), "HeapVec cap exceeded");
        assert!(index <= self.len, "index out of bounds");

        self.len += 1;
        self.array[index..self.len].rotate_right(1);
        self.array[index] = elem;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        let result = unsafe { ptr::read(self.array.as_ptr().add(index)) };
        self.array[index..self.len].rotate_left(1);
        result
    }

    pub fn push(&mut self, elem: T) {
        self.insert(self.len, elem);
    }

    pub fn pop(&mut self) -> T {
        self.remove(self.len)
    }
}

impl<T> Deref for HeapVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.array.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for HeapVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.array.as_mut_ptr(), self.len) }
    }
}
