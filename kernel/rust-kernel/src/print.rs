use crate::kputc;
use core::fmt;
use spin::Mutex;

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
        $crate::print::_print(format_args!($($arg)*))
    };
}

struct KernelConsole();

impl fmt::Write for KernelConsole{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_string(s);
        Ok(())
    }
}

static Console: Mutex<KernelConsole> = Mutex::new(KernelConsole());

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut console = Console.lock();
    console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print("\n")
    };
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}

