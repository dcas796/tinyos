#![feature(let_chains)]

fn main() {
    let bios_path = env!("BIOS_PATH");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.stdout(std::io::stdout());
    cmd.arg("-drive")
        .arg(format!("format=raw,file={bios_path}"))
        .arg("-serial")
        .arg("stdio");

    if let Some(debug) = option_env!("DEBUG")
        && debug == "1"
    {
        cmd.arg("-s").arg("-S");
    }

    let mut child = cmd.spawn().unwrap();
    let code = child.wait().unwrap();

    println!("Exit Code: {code}")
}
