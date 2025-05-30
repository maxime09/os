use crate::{inb, keyboard_interrupt, println, PIC_sendEOI, kputc};

const KEYBOARD_DATA_PORT: u16 = 0x60;


pub unsafe fn read_scancode() -> u8{
    unsafe{
        inb(KEYBOARD_DATA_PORT)
    }
}

pub fn handle_keyboard_interrupt(){
    let scancode = unsafe{read_scancode()};
    if let Some(c) = scancode_to_char(parse_scancode(scancode)){
        unsafe{kputc(c as i8)};
    }
    PIC_sendEOI(keyboard_interrupt);
}

pub enum KeyCode{
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Unknown(u8)
}

pub enum ScanCode{
    Pressed(KeyCode),
    Released(KeyCode),
}

pub fn parse_scancode(code: u8) -> ScanCode{
    use KeyCode::*;
    let key = code & (0x7f);
    let keycode = match key{
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
        _ => Unknown(key)
    };
    let scancode = if code >= 0x80{
        ScanCode::Released(keycode)
    } else{
        ScanCode::Pressed(keycode)
    };
    scancode
}

pub fn keycode_to_char(code: KeyCode) -> char{
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

pub fn scancode_to_char(code: ScanCode) -> Option<char>{
    match code{
        ScanCode::Pressed(keycode) => Some(keycode_to_char(keycode)),
        ScanCode::Released(_) => None,
    }
}