use crate::println;

pub mod pic;
pub use pic::*;
pub mod keyboard;

pub const keyboard_interrupt: u8 = 33; // 1 + offset

#[unsafe(no_mangle)]
pub extern "C" fn rust_interrupt_handler(interrupt_code: u64, error_code: u64){
    match interrupt_code as u8{
        keyboard_interrupt => {keyboard::handle_keyboard_interrupt();}
        _ => {println!("Interrupt {}, error code: {}", interrupt_code, error_code);}
    }
}

