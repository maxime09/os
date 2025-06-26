use crate::{kputc, pit::reload_pit_with_same_divisor, println};

pub mod pic;
pub use pic::*;
pub mod keyboard;

pub const double_fault: u8 = 8;
pub const page_fault: u8 = 14;
pub const PIT_Interrupt: u8 = 32;
pub const keyboard_interrupt: u8 = 33; // 1 + offset

#[unsafe(no_mangle)]
pub extern "C" fn rust_interrupt_handler(interrupt_code: u64, error_code: u64){
    match interrupt_code as u8{
        keyboard_interrupt => {keyboard::handle_keyboard_interrupt();},
        double_fault => {double_fault_handler(error_code);},
        PIT_Interrupt => {PIT_handler();},
        page_fault => {page_fault_handler();},
        _ => {panic!("Unhandled interrupt {}, error code: {}", interrupt_code, error_code);},
    }
}

pub fn double_fault_handler(error_code: u64){
    println!("Double fault: code {}", error_code);
}

pub fn PIT_handler(){
    unsafe {
        kputc(b'.' as i8);
    }
    PIC_sendEOI(PIT_Interrupt);

    reload_pit_with_same_divisor();
}

pub fn page_fault_handler(){
    let cr2 = x86_64::registers::control::Cr2::read_raw();
    panic!("Page fault at address: 0x{:x}", cr2);
}