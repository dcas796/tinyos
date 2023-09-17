use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::mem::{ManuallyDrop, needs_drop};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{mem, ptr, slice};

pub struct HeapArray<T> {
    ptr: NonNull<T>,
    len: usize,
}

impl<T> HeapArray<T> {
    pub fn new(len: usize) -> Option<Self> {
        let ptr = unsafe {
            let layout = Layout::array::<T>(len).ok()?;
            NonNull::new(alloc(layout).cast())?
        };
        Some(Self { ptr, len })
    }

    pub const fn new_with_ptr(ptr: NonNull<T>, len: usize) -> Self {
        Self { ptr, len }
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i < self.len {
            unsafe { Some(&*(self.ptr.as_ptr().add(i))) }
        } else {
            None
        }
    }
    pub fn get_mut(&self, i: usize) -> Option<&mut T> {
        if i < self.len {
            unsafe { Some(&mut *(self.ptr.as_ptr().add(i))) }
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for HeapArray<T> {
    fn drop(&mut self) {
        if needs_drop::<T>() {
            for item in self.iter_mut() {
                unsafe { ptr::drop_in_place(item); }
            }
        }
        let layout = Layout::array::<T>(self.len).unwrap();
        unsafe {
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Deref for HeapArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for HeapArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T: Clone> Clone for HeapArray<T> {
    fn clone(&self) -> Self {
        let new_array = HeapArray::new(self.len).unwrap();
        for (i, item) in self.iter().enumerate() {
            *new_array.get_mut(i).unwrap() = item.clone();
        }
        new_array
    }
}

impl<T> IntoIterator for HeapArray<T> {
    type Item = T;
    type IntoIter = HeapArrayIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let array = ManuallyDrop::new(self);
        let ptr = array.ptr;
        let len = array.len;

        unsafe {
            HeapArrayIter {
                ptr,
                len,
                start: ptr.as_ptr(),
                end: if len == 0 {
                    ptr.as_ptr()
                } else {
                    ptr.as_ptr().add(len)
                }
            }
        }
    }
}

unsafe impl<T: Send> Send for HeapArray<T> {}
unsafe impl<T: Sync> Sync for HeapArray<T> {}

pub struct HeapArrayIter<T> {
    ptr: NonNull<T>,
    len: usize,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for HeapArrayIter<T> {
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
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for HeapArrayIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for HeapArrayIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
        let layout = Layout::array::<T>(self.len).unwrap();
        unsafe {
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}
