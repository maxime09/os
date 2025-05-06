use core::ffi::*;
use crate::println;

pub mod pic;
pub use pic::*;

#[unsafe(no_mangle)]
pub extern "C" fn rust_interrupt_handler(interrupt_code: c_uint, error_code: c_uint){
    println!("Interrupt {}, error code: {}", interrupt_code, error_code);
}