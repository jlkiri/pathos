fn main() {
    println!("cargo:rustc-link-arg-bin=pathos=-Tkernel.ld");
}
