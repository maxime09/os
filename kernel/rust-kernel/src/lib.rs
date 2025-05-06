#![no_std]
#![no_main]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use core::{fmt, panic::PanicInfo};
pub mod interrupts;
pub use interrupts::*;
use x86_64::instructions::hlt;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        hlt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn test() -> !{
    println!("Hello from rust!\n");
    loop {
        hlt();
    }
}

pub fn write_string(s: &str){
    for byte in s.bytes(){
        match byte{
            0x00..0x80 => unsafe { kputc(byte as i8)}
            _ => ()
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

struct KernelConsole();

impl fmt::Write for KernelConsole{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut console = KernelConsole();
    console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print("\n")
    };
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}