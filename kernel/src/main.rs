#![feature(let_chains)]
#![feature(allocator_api)]
#![feature(exclusive_range_pattern)]
#![cfg_attr(not(test),no_std)]
#![cfg_attr(not(test),no_main)]

extern crate alloc;

use core::ptr::NonNull;
use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use crate::alloc_sys::ALLOCATOR;

mod alloc_sys;
mod vga;
mod utils;

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    initialize_allocator(&boot_info.memory_regions);

    hlt_loop();
}

fn initialize_allocator(regions: &MemoryRegions) {
    let mut usable_region: Option<MemoryRegion> = None;
    for region in regions.iter() {
        if region.kind == MemoryRegionKind::Usable {
            usable_region = Some(*region);
            break;
        }
    }
    let ptr = NonNull::new(usable_region.unwrap().start as *mut u8).unwrap();
    ALLOCATOR.initialize(ptr, usable_region.unwrap().end as usize);
}

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    hlt_loop()
}

#[cfg(test)]
fn main() {}
