#![feature(let_chains)]
#![feature(allocator_api)]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(isqrt)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code, static_mut_refs)]

extern crate alloc;

use crate::alloc_sys::ALLOCATOR;
use crate::logger::{log, logln};
use crate::vga::{VgaMode, VgaScreen};
use bootloader_api::config::Mapping;
use bootloader_api::info::MemoryRegionKind;
use bootloader_api::{BootInfo, BootloaderConfig};
use core::ptr::NonNull;
use vga::char::VgaStyle;
use vga::TEXT_SCREEN_ROWS;

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
    logln!(
        "
------------------------------------------

           WELCOME TO MY OS!!!

------------------------------------------

:)",
    );

    logln!("Initializing allocator...");
    initialize_allocator(&boot_info);

    logln!("Initializing screen...");
    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Cannot find Framebuffer, it is None.");
    let mut screen = VgaScreen::new(framebuffer).expect("Cannot initialize screen.");

    logln!("Painting screen...");
    screen.clear_screen();
    screen.mode = VgaMode::Text;

    screen.print_text(
        0,
        0,
        concat!("TinyOS Kernel ", env!("CARGO_PKG_VERSION")),
        VgaStyle::default(),
    );
    screen.print_text(
        0,
        TEXT_SCREEN_ROWS - 1,
        "© 2024 dcas796 (https://github.com/dcas796)",
        VgaStyle::default(),
    );
    screen.print_text(0, 1, "Loading OS...", VgaStyle::default());

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
    log!(
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
