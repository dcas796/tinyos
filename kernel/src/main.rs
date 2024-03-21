#![feature(let_chains)]
#![feature(allocator_api)]
#![feature(exclusive_range_pattern)]
#![feature(non_null_convenience)]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(generic_nonzero)]
#![feature(nonzero_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code, static_mut_ref)]

extern crate alloc;

use crate::alloc_sys::ALLOCATOR;
use crate::logger::logger;
use crate::vga::{VgaMode, VgaScreen};
use bootloader_api::config::Mapping;
use bootloader_api::info::{MemoryRegion, MemoryRegionKind};
use bootloader_api::{BootInfo, BootloaderConfig};
use core::cmp::min;
use core::fmt::Write;
use core::ptr::NonNull;
use vga::color::VgaColor;
use vga::pixel::VgaPixel;

mod alloc_sys;
mod logger;
mod utils;
mod vga;

#[cfg(not(test))]
static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    _ = logger().write_str(
        "
------------------------------------------

           WELCOME TO MY OS!!!

------------------------------------------

:)",
    );

    _ = logger().write_str("Initializing allocator...");
    initialize_allocator(&boot_info);

    _ = logger().write_str("Initializing screen...");
    let framebuffer = match boot_info.framebuffer.as_mut() {
        Some(f) => f,
        None => hlt_loop(),
    };
    let mut screen = VgaScreen::new(framebuffer);
    screen.mode = VgaMode::Pixels;
    screen.clear_screen();

    screen.pixel_buffer.fill(VgaPixel(VgaColor::blue()));
    screen.draw();

    hlt_loop();
}

fn initialize_allocator(boot_info: &BootInfo) {
    let usable_region = boot_info
        .memory_regions
        .iter()
        .find(|m| m.kind == MemoryRegionKind::Usable)
        .expect("Cannot find suitable free memory.");
    let start = usable_region.start;
    let end = usable_region.end;
    let offset = boot_info.physical_memory_offset.into_option().unwrap_or(0);
    let size = (end - start) as usize;
    let ptr = NonNull::new((start + offset) as *mut u8)
        .expect("Unexpected null pointer when initializing allocator.");
    unsafe { ptr.write_bytes(0, size) };
    ALLOCATOR.initialize(ptr, size);
}

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    _ = write!(
        logger(),
        "
------------------------------------------

            PANIC IN MY OS :(

------------------------------------------

{info}
"
    );
    hlt_loop();
}

#[cfg(test)]
fn main() {}
