fn main() {
    println!("cargo:rustc-link-arg-bin=interrupts=-Tkernel.ld");
}
