use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::fmt::Debug;
use core::mem::needs_drop;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{ptr, slice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapArrayError {
    LayoutError(core::alloc::LayoutError),
    AllocationError,
}

impl From<core::alloc::LayoutError> for HeapArrayError {
    fn from(value: core::alloc::LayoutError) -> Self {
        Self::LayoutError(value)
    }
}

pub struct HeapArray<T> {
    ptr: NonNull<T>,
    len: usize,
}

impl<T> HeapArray<T> {
    pub fn new(len: usize) -> Result<Self, HeapArrayError> {
        let ptr = unsafe {
            let layout = Layout::array::<T>(len)?;
            NonNull::new(alloc(layout).cast()).ok_or(HeapArrayError::AllocationError)?
        };
        Ok(Self { ptr, len })
    }

    pub const fn new_with_ptr(ptr: NonNull<T>, len: usize) -> Self {
        Self { ptr, len }
    }

    pub fn ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn ptr_mut(&mut self) -> *mut T {
        self.ptr.as_ptr()
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
                unsafe {
                    ptr::drop_in_place(item);
                }
            }
        }
        let layout = Layout::array::<T>(self.len)
            .expect("Could not drop HeapArray<T>: Cannot create the necessary layout for deallocating its base pointer.");
        unsafe {
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Deref for HeapArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for HeapArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T: Clone> Clone for HeapArray<T> {
    fn clone(&self) -> Self {
        let mut new_array =
            HeapArray::new(self.len).expect("Cannot create a new HeapArray while cloning another.");
        for (i, item) in self.iter().enumerate() {
            new_array[i] = item.clone();
        }
        new_array
    }
}

impl<T: Debug> Debug for HeapArray<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "[")?;
        for elem in self.iter() {
            writeln!(f, "\t{:?}", elem)?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}
