mod block;
mod map;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;
use crate::alloc_sys::map::MemoryMap;

#[cfg_attr(not(test),global_allocator)]
pub static ALLOCATOR: SystemAllocator = SystemAllocator::new();

pub struct SystemAllocator {
    map: UnsafeCell<MemoryMap>,
}

impl SystemAllocator {
    pub const fn new() -> Self {
        Self {
            map: UnsafeCell::new(MemoryMap::new()),
        }
    }
}

unsafe impl GlobalAlloc for SystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.map.get().as_mut() {
            Some(map) => map
                .alloc(layout.size(), layout.align(), false)
                .unwrap_or(null_mut()),
            None => null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        match self.map.get().as_mut() {
            Some(map) => map.dealloc(ptr),
            None => {}
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        match self.map.get().as_mut() {
            Some(map) => map
                .alloc(layout.size(), layout.align(), true)
                .unwrap_or(null_mut()),
            None => null_mut()
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        match self.map.get().as_mut() {
            Some(map) => map
                .realloc(ptr, new_size, layout.align())
                .unwrap_or(null_mut()),
            None => null_mut()
        }
    }
}

unsafe impl Sync for SystemAllocator {}
