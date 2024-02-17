#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]
// #![feature(custom_test_frameworks)]

pub mod serial;
