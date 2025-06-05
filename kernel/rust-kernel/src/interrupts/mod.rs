use crate::println;

pub mod pic;
pub use pic::*;
pub mod keyboard;

pub const double_fault: u8 = 8;

pub const keyboard_interrupt: u8 = 33; // 1 + offset

#[unsafe(no_mangle)]
pub extern "C" fn rust_interrupt_handler(interrupt_code: u64, error_code: u64){
    match interrupt_code as u8{
        keyboard_interrupt => {keyboard::handle_keyboard_interrupt();},
        double_fault => {double_fault_handler(error_code);}
        _ => {println!("Interrupt {}, error code: {}", interrupt_code, error_code);}
    }
}

pub fn double_fault_handler(error_code: u64){
    println!("Double fault: code {}", error_code);
}

