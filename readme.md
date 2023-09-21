# TinyOS

An experimental x86_64 OS written in Rust. Inspired by early MS-DOS and Unix systems.

## Concept

It has no graphical user interface, only text on the screen. You can run programs through the command line,
however, the kernel **does not have hard drive support** yet (it might be later on added as another integrated driver).

It implements its own memory allocator; screen, keyboard and mouse drivers; and an interpreter program that is launched
when booted. It is still at a very early stage and has yet not been tested on real hardware.

## Architecture
[TinyOS.pdf](images/TinyOS.pdf)

## Prerequisites

Before compiling the project, you must download the nightly version of Cargo.
It is recommended to use `rustup`, as it makes the whole process much easier.

To install the nightly version of Cargo, run: 
```sh
rustup default nightly
```

As you will be cross-compiling, you need to download the `x86_64-unknown-none` target, through `rustup`:
```sh
rustup target add x86_64-unknown-none
```
You also need the `llvm-tools-preview` component, which can be installed through:
```sh
rustup component add llvm-tools-preview
```

To run the project, you need QEMU. To install it, run:
- macOS: `brew install qemu`
- Ubuntu/Debian-based distros: `sudo apt install qemu-system`

### On ARM Macs

To test the project, you will need to compile it in the x86_64 architecture, as it will fail otherwise.
But first, you need to add the x86_64 version of the Apple Darwin toolchain:
```sh
rustup target add x86_64-apple-darwin
```

## Compiling the OS

The OS is separated into three parts: the **bootloader**, the **kernel**, and the **_Interp_ program**.

The bootloader is not in the scope of this project, as it would have made it 10x harder.
It is from [rust-osdev/bootloader](https://github.com/rust-osdev/bootloader).

- To compile the entire project, just run: `cargo build`

- To compile the kernel or _Interp_ program individually, run:
`cd kernel` or `cd interp` and then `cargo build --target x86_64-unknown-none`

## Booting the OS

To run the OS in a virtual environment, run: `cargo run`.
This will spawn a new QEMU window with your compiled OS.

If you want the compiled boot image, build the project, and the boot image will be located in `target/debug/build/tinyos-*/out/bios.img`

## Testing

This project includes tests. To test the project, run:
- macOS: `RUST_BACKTRACE=1 cargo test --target x86_64-apple-darwin -- --nocapture`
- Linux: `RUST_BACKTRACE=1 cargo test --target x86_64-unknown-linux-gnu -- --nocapture`

---

Made by [dcas796](https://dcas796.github.com/)

