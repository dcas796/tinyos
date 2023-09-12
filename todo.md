# TODO

## Kernel
### Minor
- Implement the `Allocator` trait for `SystemAllocator`
- Make the buffer in `MemoryMap` to be the same size as the RAM.
- Make `MemoryBlocksIter` not use the static `ALLOCATOR` variable to access `MemoryMap`, preferably use a reference instead.

### Major

- VGA driver
- Keyboard driver
- Mouse driver

## Interp program

- Make it
