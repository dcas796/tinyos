mod block;
mod map;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{NonNull, null_mut};
use crate::alloc_sys::map::MemoryMap;

#[cfg_attr(not(test),global_allocator)]
pub static ALLOCATOR: SystemAllocator = SystemAllocator::new();

pub struct SystemAllocator {
    map: UnsafeCell<Option<MemoryMap>>,
}

impl SystemAllocator {
    pub const fn new() -> Self {
        Self {
            map: UnsafeCell::new(None),
        }
    }

    pub fn initialize(&self, ptr: NonNull<u8>, len: usize) {
        unsafe {
            match self.map.get().as_mut() {
                Some(opt) if opt.is_some() => {},
                Some(opt) => *opt = Some(MemoryMap::new(ptr, len)),
                None => {}
            }
        }
    }
}

unsafe impl GlobalAlloc for SystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.map.get().as_mut() {
            Some(Some(map)) => map
                .alloc(layout.size(), layout.align(), false)
                .unwrap_or(null_mut()),
            _ => null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        match self.map.get().as_mut() {
            Some(Some(map)) => map.dealloc(ptr),
            _ => {}
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        match self.map.get().as_mut() {
            Some(Some(map)) => map
                .alloc(layout.size(), layout.align(), true)
                .unwrap_or(null_mut()),
            _ => null_mut()
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        match self.map.get().as_mut() {
            Some(Some(map)) => map
                .realloc(ptr, new_size, layout.align())
                .unwrap_or(null_mut()),
            _ => null_mut()
        }
    }
}

unsafe impl Sync for SystemAllocator {}
