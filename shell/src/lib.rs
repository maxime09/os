#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn shell_main(print: extern "C" fn(*const u8)) -> !{
    let str = b"Hello from rust!".as_ptr();
    print(str);
    loop{}
}

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    loop{}
}