#![feature(let_chains)]
#![feature(allocator_api)]
#![cfg_attr(not(test),no_std)]
#![cfg_attr(not(test),no_main)]

extern crate alloc;

use alloc::vec;

mod alloc_sys;

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    // Test the allocator
    let mut vec = vec!["hello"];
    vec.push("world");
    vec.push("!");
    vec.pop();

    hlt_loop();
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
