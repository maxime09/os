use crate::{inb, keyboard_interrupt, println, PIC_sendEOI};

const KEYBOARD_DATA_PORT: u16 = 0x60;


pub unsafe fn read_scancode() -> u8{
    unsafe{
        inb(KEYBOARD_DATA_PORT)
    }
}

pub fn handle_keyboard_interrupt(){
    let scancode = unsafe{read_scancode()};
    println!("Received keyboard interrupt, code: {}", scancode);
    PIC_sendEOI(keyboard_interrupt);
}