use std::{
    io::{prelude::BufRead, BufReader},
    process::{Command, Stdio},
};

// qemu-system-riscv64 --machine virt --serial stdio --monitor none \
//         --bios {{bin}} --nographic \
//         -d guest_errors,unimp -D log.txt -m 128M

fn main() {
    let bin = std::env::args()
        .nth(1)
        .expect("No binary file was provided.");

    println!("Using binary file: {}", bin);

    let mut cmd = Command::new("qemu-system-riscv64");
    cmd.args([
        "--machine",
        "virt",
        "--serial",
        "stdio",
        "--monitor",
        "none",
        "--bios",
        &bin,
        "--nographic",
        "-d",
        "guest_errors,unimp",
        "-D",
        "log.txt",
        "-m",
        "128M",
    ])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit());

    println!("{cmd:?}");

    let mut child = cmd.spawn().expect("Failed to start QEMU");

    // {
    //     let stdout = child.stdout.as_mut().unwrap();
    //     let stdout_reader = BufReader::new(stdout);
    //     let stdout_lines = stdout_reader.lines();

    //     for line in stdout_lines {
    //         println!("Read: {:?}", line);
    //     }
    // }

    child.wait().expect("Failed to wait for QEMU to exit");
}
