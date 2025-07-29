use crate::{apic, find_page_entry, pit, println};

pub mod pic;
pub use pic::*;
pub mod keyboard;
pub mod syscall;

pub const double_fault: u8 = 8;
pub const page_fault: u8 = 14;
pub const PIT_Interrupt: u8 = 32;
pub const keyboard_interrupt: u8 = 33; // 1 + offset
pub const first_pic_spurious: u8 = 39;
pub const apic_keyboard: u8 = 49;
pub const PIT_APIC: u8 = 48;
pub const APIC_TIMER: u8 = 50;
pub const division_by_0: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn rust_interrupt_handler(interrupt_code: u64, error_code: u64){
    match interrupt_code as u8{
        keyboard_interrupt => {keyboard::handle_keyboard_interrupt();},
        double_fault => {double_fault_handler(error_code);},
        PIT_Interrupt => {PIT_handler();},
        page_fault => {page_fault_handler(error_code);},
        first_pic_spurious => {PIC_sendEOI(first_pic_spurious);},
        apic_keyboard => {keyboard::handle_apic_keyboard_interrupt()},
        PIT_APIC => {pit::interrupt_apic()},
        APIC_TIMER => {handle_apic_timer();}
        division_by_0 => {panic!("Division by 0")},
        _ => {println!("Unhandled interrupt {}, error code: {}. Ignoring it.", interrupt_code, error_code);},
    }
}

pub fn double_fault_handler(error_code: u64){
    println!("Double fault: code {}", error_code);
}

pub fn PIT_handler(){
    PIC_sendEOI(PIT_Interrupt);
}

pub fn page_fault_handler(error_code: u64){
    let cr2 = x86_64::registers::control::Cr2::read_raw();
    println!("Page fault at address: 0x{:x}", cr2);
    let (frame, cr3) = x86_64::registers::control::Cr3::read_raw();
    println!("cr3: 0x{:x}, frame: {:?}", cr3, frame);

    let paging_entry = unsafe {
        find_page_entry(cr2 as usize)
    };
    println!("Paging entry: 0x{:x}", paging_entry);
    println!("Error code: 0x{:x}", error_code);
    panic!("Not able to recover (yet).");
}

pub fn handle_apic_timer(){
    apic::send_EOI();
}

