use core::{cmp::Ordering, mem::align_of, num::NonZero};

pub const MEMORY_BLOCK_ALIGN: NonZero<usize> =
    unsafe { NonZero::new_unchecked(align_of::<MemoryBlock>()) };

#[derive(Debug, Copy, Clone)]
pub struct MemoryBlock {
    pub is_active: bool,
    pub start: usize,
    pub size: usize,
}

impl MemoryBlock {
    pub fn new(start: usize, size: usize) -> Self {
        Self {
            is_active: true,
            start,
            size,
        }
    }

    pub const fn null() -> Self {
        Self {
            is_active: false,
            start: 0,
            size: 0,
        }
    }

    pub fn end(&self) -> usize {
        self.start + self.size
    }
}

impl PartialEq for MemoryBlock {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

impl Eq for MemoryBlock {}

impl PartialOrd for MemoryBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start.partial_cmp(&other.start)
    }
}

impl Ord for MemoryBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}
