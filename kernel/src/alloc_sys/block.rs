use core::cmp::Ordering;
use core::marker::PhantomData;
use core::slice::Windows;
use crate::alloc_sys::ALLOCATOR;
use crate::alloc_sys::map::MAX_BLOCKS;

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
            size
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



#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MemoryBlocks {
    pub(self) blocks: [MemoryBlock; MAX_BLOCKS],
    length: usize,
}

impl MemoryBlocks {
    pub const fn new() -> Self {
        Self {
            blocks: [MemoryBlock::null(); MAX_BLOCKS],
            length: 0,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn first(&self) -> Option<&MemoryBlock> {
        self.blocks.get(0)
    }

    pub fn first_mut(&mut self) -> Option<&mut MemoryBlock> {
        self.blocks.get_mut(0)
    }

    pub fn last(&self) -> Option<&MemoryBlock> {
        if self.length == 0 { return None; }
        self.blocks.get(self.length - 1)
    }

    pub fn last_mut(&mut self) -> Option<&mut MemoryBlock> {
        self.blocks.get_mut(self.length - 1)
    }

    pub fn get(&self, n: usize) -> Option<&MemoryBlock> {
        if n < self.length {
            self.blocks.get(n)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, n: usize) -> Option<&mut MemoryBlock> {
        if n < self.length {
            self.blocks.get_mut(n)
        } else {
            None
        }
    }

    pub fn insert(&mut self, block: MemoryBlock) {
        if self.length == 0 {
            self.blocks[0] = block;
        } else {
            match self.binary_search(&block) {
                Ok(_) => {},
                Err(i) => {
                    self.blocks[i..self.length].rotate_right(1);
                    self.blocks[i] = block;
                },
            };
        }
        self.length += 1;
    }

    pub fn remove(&mut self, block: &MemoryBlock) {
        match self.iter().position(|b| b == block) {
            Some(i) => {
                self.blocks[i..self.length].rotate_left(1);
                self.blocks.last_mut().unwrap().is_active = false;
                self.length -= 1;
            }
            None => {}
        }
    }

    pub fn binary_search(&self, block: &MemoryBlock) -> Result<usize, usize> {
        self.binary_search_by(|b| b.cmp(block))
    }

    pub fn binary_search_by<F: FnMut(&MemoryBlock) -> Ordering>(
        &self,
        mut f: F
    ) -> Result<usize, usize> {
        let mut size = self.length;
        let mut left = 0;
        let mut right = size;
        while left < right {
            let mid = left + size / 2;
            let cmp = f(unsafe { self.blocks.get_unchecked(mid) });
            if cmp == Ordering::Less {
                left = mid + 1;
            } else if cmp == Ordering::Greater {
                right = mid;
            } else {
                return Ok(mid);
            }
            size = right - left;
        }
        Err(left)
    }

    pub fn windows(&self, size: usize) -> BlockWindows {
        let windows = self.blocks.windows(size);
        BlockWindows::new(windows, size, self.length)
    }

    pub fn iter(&self) -> MemoryBlocksIter {
        MemoryBlocksIter {
            index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MemoryBlocksIter<'a> {
    index: usize,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> Iterator for MemoryBlocksIter<'a> {
    type Item = &'a MemoryBlock;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // TODO: This needs to be more safe...
            let blocks = &ALLOCATOR.map.get().as_ref()?.blocks;
            if self.index < blocks.length {
                Some(&blocks.blocks[self.index])
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockWindows<'a> {
    windows: Windows<'a, MemoryBlock>,
    size: usize,
    length: usize,
}

impl<'a> BlockWindows<'a> {
    pub fn new(windows: Windows<'a, MemoryBlock>, size: usize, length: usize) -> Self {
        Self {
            windows,
            size,
            length,
        }
    }
}

impl<'a> Iterator for BlockWindows<'a> {
    type Item = &'a [MemoryBlock];

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 { return None; }
        let window = self.windows.next()?;
        let (length, overflow) = self.length.overflowing_add(self.size);
        if overflow {
            None
        } else {
            self.length = length;
            Some(window)
        }
    }
}
