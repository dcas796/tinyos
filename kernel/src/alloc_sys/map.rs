use crate::alloc_sys::block::MemoryBlock;
use crate::utils::heap_array::HeapArray;
use crate::utils::heap_vec::HeapVec;
use crate::utils::non_zero_rem::NonZeroRem;
use core::mem;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

pub const MAX_ALIGN: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(4096) };
pub const BLOCKS_BUFFER_FRACTION: f64 = 0.1;

pub struct MemoryMap {
    buffer: HeapArray<u8>,
    blocks: HeapVec<MemoryBlock>,
}

impl MemoryMap {
    // pub fn _new(ptr: NonNull<u8>, len: usize) -> Self {
    //     let ptr_addr = ptr.as_ptr() as usize;
    //     let aligned_diff = MAX_ALIGN - (ptr_addr % MAX_ALIGN);
    //     let aligned_ptr_addr = ptr_addr + aligned_diff;
    //     assert!(
    //         aligned_ptr_addr < (ptr_addr + len),
    //         "Insufficient size for ALLOCATOR"
    //     );
    //     let len_aligned = len - aligned_diff;
    //     let aligned_ptr = NonNull::new(aligned_ptr_addr as *mut u8).unwrap();
    //     let blocks_len = (BLOCKS_BUFFER_FRACTION * len_aligned as f64) as usize;

    //     let blocks_ptr = unsafe {
    //         let ptr = aligned_ptr.as_ptr().add(len_aligned - blocks_len);
    //         NonNull::new(ptr.cast::<MemoryBlock>()).unwrap()
    //     };
    //     Self {
    //         buffer: HeapArray::new_with_ptr(aligned_ptr, len_aligned - blocks_len),
    //         blocks: HeapVec::new_with_ptr(blocks_ptr, blocks_len / mem::size_of::<MemoryBlock>()),
    //     }
    // }

    pub fn new(ptr: NonNull<u8>, len: usize) -> Self {
        let ptr_addr = ptr.addr();
        let aligned_diff = MAX_ALIGN.get() - (ptr_addr.non_zero_rem(MAX_ALIGN)).get();
        let aligned_ptr_addr = ptr_addr
            .checked_add(aligned_diff)
            .expect("Overflowed usize when calculating aligned MemoryMap base pointer");
        assert!(
            aligned_ptr_addr.get() < (ptr_addr.get() + len),
            "Insufficient size for ALLOCATOR"
        );
        let len_aligned = len - aligned_diff;
        let aligned_ptr = NonNull::dangling().with_addr(aligned_ptr_addr);
        let blocks_bytes_len = (BLOCKS_BUFFER_FRACTION * (len_aligned as f64)) as usize;
        let blocks_ptr = unsafe {
            aligned_ptr
                .add(len_aligned - blocks_bytes_len)
                .cast::<MemoryBlock>()
        };
        Self {
            buffer: HeapArray::new_with_ptr(aligned_ptr, len_aligned - blocks_bytes_len),
            blocks: HeapVec::new_with_ptr(
                blocks_ptr,
                blocks_bytes_len / mem::size_of::<MemoryBlock>(),
            ),
        }
    }

    pub unsafe fn alloc(&mut self, size: usize, align: usize, zeroed: bool) -> Option<*mut u8> {
        if size >= self.buffer.len() || align > MAX_ALIGN.get() {
            return None;
        }

        let padded_size = size + align - 1;
        let block = if let Some(block) = self.blocks.last() {
            let end = block.end();
            let free_space = self.buffer.len() - end;
            if free_space >= padded_size {
                let start = end + align - (end % align);
                MemoryBlock::new(start, size)
            } else {
                if self.blocks.first().unwrap().start >= padded_size {
                    MemoryBlock::new(0, size)
                } else if (self.buffer.len() - self.blocks.last().unwrap().end()) >= padded_size {
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

        self.insert_block(block);
        if zeroed {
            for i in block.start..block.end() {
                self.buffer[i] = 0;
            }
        }
        Some(self.ptr_for_start(block.start))
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {
        if let Some(start) = self.start_for_ptr(ptr) {
            self.remove_block(MemoryBlock::new(start, 0));
        }
    }
    pub unsafe fn realloc(&mut self, ptr: *mut u8, size: usize, align: usize) -> Option<*mut u8> {
        let start = self.start_for_ptr(ptr)?;
        let pos = self.blocks.iter().position(|block| block.start == start)?;
        let free_space_end = if let Some(next_block) = self.blocks.get(pos + 1) {
            next_block.start
        } else {
            self.buffer.len()
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
        assert!(start < self.buffer.len());
        self.buffer.as_mut_ptr().add(start)
    }

    unsafe fn start_for_ptr(&self, ptr: *const u8) -> Option<usize> {
        if ptr < self.buffer.as_ptr() {
            return None;
        }
        Some(ptr.sub(self.buffer.as_ptr() as usize) as usize)
    }

    fn insert_block(&mut self, block: MemoryBlock) {
        let position = match self.blocks.binary_search(&block) {
            Ok(_) => panic!("Cannot allocate the another block to the same pointer"),
            Err(i) => i,
        };
        self.blocks.insert(position, block);
    }

    fn remove_block(&mut self, block: MemoryBlock) {
        let position = match self.blocks.binary_search(&block) {
            Ok(i) => i,
            Err(_) => return,
        };
        self.blocks.remove(position);
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc_sys::ALLOCATOR;
    use crate::utils::heap_array::HeapArray;
    use alloc::alloc::alloc;
    use core::alloc::{GlobalAlloc, Layout};
    use core::mem::ManuallyDrop;
    use core::ptr::NonNull;

    const ALLOCATOR_SIZE: usize = 10_000_000;

    unsafe fn initialize_allocator() {
        let alloc_layout = Layout::from_size_align_unchecked(ALLOCATOR_SIZE, 4096);
        let alloc_ptr = NonNull::new(alloc(alloc_layout)).unwrap();
        ALLOCATOR.initialize(alloc_ptr, ALLOCATOR_SIZE);
    }

    #[test]
    fn align_test() {
        unsafe {
            println!();
            initialize_allocator();
            let layout = Layout::from_size_align(50, 16).unwrap();
            println!(
                "Buffer ptr:\t{:#018x}",
                ALLOCATOR
                    .map
                    .get()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    // .get().unwrap()
                    .unwrap()
                    .buffer
                    .as_ptr() as usize
            );
            let my_ptr = ALLOCATOR.alloc(layout);
            let is_aligned = if my_ptr as usize % layout.align() == 0 {
                true
            } else {
                false
            };
            println!(
                "My ptr:\t\t{:#018x} is{}aligned",
                my_ptr as usize,
                if is_aligned { " " } else { " not " }
            );
            assert!(is_aligned);
        }
    }

    #[test]
    fn heap_array_test() {
        unsafe {
            println!();
            initialize_allocator();
            const ARRAY_LEN: usize = 30_000;
            let layout = Layout::array::<u8>(ARRAY_LEN).unwrap();
            let ptr = NonNull::new(ALLOCATOR.alloc(layout))
                .expect("Returned a null pointer from allocator");
            let _array = HeapArray::new_with_ptr(ptr, ARRAY_LEN);
            // Do not drop HeapArray, global_allocator is not our allocator
            let mut array = ManuallyDrop::new(_array);
            for (i, mut byte) in array.iter_mut().enumerate() {
                *byte = i as u8;
            }
            for (i, byte) in array.iter().enumerate() {
                assert_eq!(*byte, i as u8, "Error in i: {i}");
            }
        }
    }
}
