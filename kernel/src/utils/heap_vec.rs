use alloc::alloc::dealloc;
use core::ops::{Deref, DerefMut};
use core::{mem, ptr, slice};
use core::alloc::Layout;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;
use crate::utils::heap_array::HeapArray;

#[derive(Clone)]
pub struct HeapVec<T> {
    array: HeapArray<T>,
    len: usize,
}

impl<T> HeapVec<T> {
    pub fn new(cap: usize) -> Option<Self> {
        Some(Self {
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
        let elem_mut_opt = self.array.get_mut(index);
        let elem_mut = elem_mut_opt.unwrap();
        *elem_mut = elem;
        // *self.array.get_mut(index).unwrap() = elem;
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
        unsafe {
            slice::from_raw_parts(self.array.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for HeapVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.array.as_mut_ptr(), self.len)
        }
    }
}

impl<T> IntoIterator for HeapVec<T> {
    type Item = T;
    type IntoIter = HeapVecIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut vec = ManuallyDrop::new(self);

        let ptr = NonNull::new(vec.as_mut_ptr()).unwrap();

        unsafe {
            HeapVecIter {
                buf: ptr,
                cap: vec.cap(),
                start: ptr.as_ptr(),
                end: if vec.cap() == 0 {
                    ptr.as_ptr()
                } else {
                    ptr.as_ptr().add(vec.len())
                },
            }
        }
    }
}

pub struct HeapVecIter<T> {
    buf: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for HeapVecIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) /
            mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for HeapVecIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.end);
                self.end = self.end.offset(-1);
                Some(result)
            }
        }
    }
}

impl<T> Drop for HeapVecIter<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            for _ in &mut *self {}
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.buf.as_ptr() as *mut u8, layout)
            }
        }
    }
}
