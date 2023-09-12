fn main() {
    let bios_path = env!("BIOS_PATH");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));

    let mut child = cmd.spawn().unwrap();
    let code = child.wait().unwrap();
    println!("Exit Code: {code}")
}
