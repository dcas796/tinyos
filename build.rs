use std::fs;
use std::path::PathBuf;

const KERNEL_FILE_PATH: &str = "target/kernel";
const OUT_FILE_NAME: &str = "bios.img";

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let bios_path = out_dir.join(OUT_FILE_NAME);
    bootloader::BiosBoot::new(&kernel).create_disk_image(&bios_path).unwrap();

    fs::copy(kernel, PathBuf::from(KERNEL_FILE_PATH)).unwrap();

    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
