use core::mem::MaybeUninit;

use alloc::collections::vec_deque::VecDeque;
use spin::Mutex;

use crate::{PIC_sendEOI, apic, inb, io_wait, keyboard_interrupt, kputc, outb};

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_COMMAND_PORT: u16 = 0x64;
const ACK: u8 = 0xFA;
const RESEND: u8 = 0xFE;
const RESET: u8 = 0xFF;
const ENABLE_SCANNING: u8 = 0x4;
const OUTPUT_STATUS: u8 = 1;
const INPUT_STATUS: u8 = 1 << 1;
const SYSTEM_FLAG: u8 = 1 << 2;
const COMMAND_DATA: u8 = 1 << 3;

pub unsafe fn read_scancode() -> u8 {
    unsafe { inb(KEYBOARD_DATA_PORT) }
}

pub unsafe fn wait_and_read() -> u8 {
    unsafe {
        wait_for_output();
        read_scancode()
    }
}

pub unsafe fn read_status() -> u8 {
    unsafe { inb(KEYBOARD_COMMAND_PORT) }
}

pub unsafe fn write_command_port(data: u8) {
    unsafe {
        outb(KEYBOARD_COMMAND_PORT, data);
    }
}

pub unsafe fn wait_for_input() {
    unsafe {
        while (read_status() & INPUT_STATUS) != 0 {
            io_wait();
        }
    }
}

pub unsafe fn send_command_raw(command: u8) {
    unsafe {
        wait_for_input();
        write_command_port(command);
    }
}

pub unsafe fn wait_for_output() {
    unsafe {
        while (read_status() & OUTPUT_STATUS) != 1 {
            io_wait();
        }
    }
}

pub unsafe fn send_command_with_ACK(command: u8) {
    unsafe {
        send_command_raw(command);
        wait_for_output();
        while read_scancode() == RESEND {
            wait_for_input();
            write_command_port(command);
            wait_for_output();
        }
    }
}

pub fn handle_keyboard_interrupt() {
    let scancode = unsafe { read_scancode() };
    if let Some(c) = scancode_to_char(parse_scancode(scancode)) {
        unsafe { kputc(c as i8) };
    }
    PIC_sendEOI(keyboard_interrupt);
}

pub fn handle_apic_keyboard_interrupt() {
    let scancode = unsafe { read_scancode() };
    push_input(scancode);
    apic::send_EOI();
}

pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Unknown(u8),
}

pub enum ScanCode {
    Pressed(KeyCode),
    Released(KeyCode),
}

pub fn parse_scancode(code: u8) -> ScanCode {
    use KeyCode::*;
    let key = code & (0x7f);
    let keycode = match key {
        0x10 => A,
        0x11 => Z,
        0x12 => E,
        0x13 => R,
        0x14 => T,
        0x15 => Y,
        0x16 => U,
        0x17 => I,
        0x18 => O,
        0x19 => P,
        0x1E => Q,
        0x1F => S,
        0x20 => D,
        0x21 => F,
        0x22 => G,
        0x23 => H,
        0x24 => J,
        0x25 => K,
        0x26 => L,
        0x27 => M,
        0x2C => W,
        0x2D => X,
        0x2E => C,
        0x2F => V,
        0x30 => B,
        0x31 => N,
        _ => Unknown(key),
    };
    let scancode = if code >= 0x80 {
        ScanCode::Released(keycode)
    } else {
        ScanCode::Pressed(keycode)
    };
    scancode
}

pub fn keycode_to_char(code: KeyCode) -> char {
    use KeyCode::*;
    match code {
        A => 'a',
        B => 'b',
        C => 'c',
        D => 'd',
        E => 'e',
        F => 'f',
        G => 'g',
        H => 'h',
        I => 'i',
        J => 'j',
        K => 'k',
        L => 'l',
        M => 'm',
        N => 'n',
        O => 'o',
        P => 'p',
        Q => 'q',
        R => 'r',
        S => 's',
        T => 't',
        U => 'u',
        V => 'v',
        W => 'w',
        X => 'x',
        Y => 'y',
        Z => 'z',
        Unknown(_) => '?',
    }
}

pub fn scancode_to_char(code: ScanCode) -> Option<char> {
    match code {
        ScanCode::Pressed(keycode) => Some(keycode_to_char(keycode)),
        ScanCode::Released(_) => None,
    }
}

pub unsafe fn clear_buffer() {
    unsafe {
        read_scancode();
    }
}

pub unsafe fn set_configuration_byte(config_byte: u8) {
    unsafe {
        send_command_raw(0x60);
        send_command_raw(config_byte);
    }
}

pub fn reset() {
    unsafe {
        send_command_raw(0xAD);
        send_command_raw(0xA7);
        clear_buffer();
        set_configuration_byte(0b00000100);
        send_command_raw(0xAA);
        if wait_and_read() != 0x55 {
            panic!("Keyboard self test failed");
        }
        send_command_raw(0xAE);
        set_configuration_byte(0b00000101);
        send_command_raw(0xFF);
        io_wait();
        clear_buffer();
        io_wait();
        clear_buffer();
    }
}

pub fn init() {
    KEYBOARD_BUFFER.lock().write(VecDeque::new());
}

// This should only be accessed by the bootstrap processor
static KEYBOARD_BUFFER: Mutex<MaybeUninit<VecDeque<u8>>> = Mutex::new(MaybeUninit::uninit());

pub fn pop_input() -> Option<u8> {
    unsafe { KEYBOARD_BUFFER.lock().assume_init_mut().pop_front() }
}

pub fn push_input(input: u8) {
    unsafe {
        KEYBOARD_BUFFER.lock().assume_init_mut().push_back(input);
    }
}
