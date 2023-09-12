use crate::alloc_sys::block::{MemoryBlock, MemoryBlocks};

pub const MAX_ALIGN: usize = 4096;
pub const MEMORY_SIZE: usize = 4 * 1000 * 255;
pub const MAX_BLOCKS: usize = MEMORY_SIZE;

#[repr(C, align(4096))]
pub struct MemoryMap {
    buffer: [u8; MEMORY_SIZE],
    pub(crate) blocks: MemoryBlocks,
}

impl MemoryMap {
    pub const fn new() -> Self {
        Self {
            buffer: [0; MEMORY_SIZE],
            blocks: MemoryBlocks::new(),
        }
    }

    pub unsafe fn alloc(&mut self, size: usize, align: usize, zeroed: bool) -> Option<*mut u8> {
        if size > MEMORY_SIZE || align > MAX_ALIGN {
            return None;
        }

        let padded_size = size + align - 1;
        let block = if let Some(block) = self.blocks.last() {
            let end = block.end();
            let free_space = MEMORY_SIZE - end;
            if free_space >= padded_size {
                let start = end + align - (end % align);
                MemoryBlock::new(start, size)
            } else {
                if self.blocks.first().unwrap().start >= padded_size {
                    MemoryBlock::new(0, size)
                } else if (MEMORY_SIZE - self.blocks.last().unwrap().end()) >= padded_size {
                    let end = self.blocks.last().unwrap().end();
                    let start = end + align - (end % align);
                    MemoryBlock::new(start, size)
                } else {
                    let mut block = MemoryBlock::null();
                    for block_pair in self.blocks.windows(2) {
                        let inter_size = block_pair[1].start - block_pair[0].end();
                        if inter_size >= padded_size {
                            let end = block_pair[0].end();
                            let start = end + align - (end % align);
                            block = MemoryBlock::new(start, size);
                        }
                    }
                    block
                }
            }
        } else {
            MemoryBlock::new(0, size)
        };


        self.blocks.insert(block);
        if zeroed {
            for i in block.start..block.end() {
                self.buffer[i] = 0;
            }
        }
        Some(self.ptr_for_start(block.start))
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {
        if let Some(start) = self.start_for_ptr(ptr) {
            self.blocks.remove(&MemoryBlock::new(start, 0));
        }
    }
    pub unsafe fn realloc(&mut self, ptr: *mut u8, size: usize, align: usize) -> Option<*mut u8> {
        let start = self.start_for_ptr(ptr)?;
        let pos = self.blocks.iter().position(|block| block.start == start)?;
        let free_space_end = if let Some(next_block) = self.blocks.get(pos + 1) {
            next_block.start
        } else {
            MEMORY_SIZE
        };
        let block = self.blocks.get(pos)?;
        let block_size = block.size;
        if free_space_end >= size {
            let new_ptr = self.ptr_for_start(block.start);
            let blocks_mut = self.blocks.get_mut(pos)?;
            blocks_mut.size = size;
            Some(new_ptr)
        } else {
            let new_ptr = self.alloc(size, align, false)?;
            new_ptr.copy_from(ptr, block_size);
            self.dealloc(ptr);
            Some(new_ptr)
        }
    }

    unsafe fn ptr_for_start(&mut self, start: usize) -> *mut u8 {
        assert!(start < MEMORY_SIZE);
        self.buffer.as_mut_ptr().add(start)
    }

    unsafe fn start_for_ptr(&mut self, ptr: *mut u8) -> Option<usize> {
        if ptr < self.buffer.as_mut_ptr() { return None; }
        Some(ptr.sub(self.buffer.as_mut_ptr() as usize) as usize)
    }
}

#[cfg(test)]
mod tests {
    use alloc::alloc::alloc;
    use core::alloc::{GlobalAlloc, Layout};
    use crate::alloc_sys::ALLOCATOR;

    #[test]
    fn alloc_test() {
        unsafe {
            let layout = Layout::from_size_align(50, 16).unwrap();
            println!("Buffer ptr:\t{:#018x}", ALLOCATOR.map.get().as_mut().unwrap().buffer.as_mut_ptr() as usize);
            let my_ptr = ALLOCATOR.alloc(layout);
            println!("My ptr:\t\t{:#018x}", my_ptr as usize);
            assert_eq!(my_ptr as usize % layout.align(), 0);
            let std_ptr = alloc(layout);
            println!("Std ptr:\t{:#018x}", std_ptr as usize);
        }
    }
}