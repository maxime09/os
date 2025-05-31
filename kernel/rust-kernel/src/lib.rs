#![no_std]
#![no_main]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate alloc;

use core::{ffi::c_int, fmt, panic::PanicInfo, slice};
pub mod interrupts;
use alloc::{boxed::Box, string::String};
use fs::vfs;
pub use interrupts::*;
use x86_64::instructions::hlt;
pub mod fs;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        hlt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_kmain(initrd_ptr: *const core::ffi::c_void, initrd_size: usize) -> !{
    println!("Hello from rust!");
    


    let data = unsafe{Box::from_raw(slice::from_raw_parts_mut(initrd_ptr as *mut u8, initrd_size))};

    let headers = fs::ustar::parse_file(&data);

    let vfs = fs::ustar::headers_to_fs(headers, data);

    let path_test = vfs::PathBuf::from("test3/test4");
    let mountpoint = vfs.get_mountpoint().unwrap();
    let node = vfs.find(path_test).unwrap();
    let data = vfs.read(mountpoint, node, 0, 1024).unwrap();
    let data_str = str::from_utf8(&data).unwrap();
    println!("\n\n{}", data_str);

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

#[global_allocator]
static ALLOCATOR: talc::Talck<spin::Mutex<()>, talc::ErrOnOom> = talc::Talc::new(talc::ErrOnOom).lock();

const kernel_heap_size: usize = 4*1024*1024; // 4 Mb
const heap_virtual_addr_start: usize = 0x1_000_000;
const page_size: usize = 4096;
const PTE_PRESENT: c_int = 1;
const PTE_READ_WRITE: c_int = 2;

#[unsafe(no_mangle)]
pub extern "C" fn init_alloc(){
    let page_count = kernel_heap_size / page_size;
    
    let mut virt_addr = heap_virtual_addr_start;
    for _ in 0..page_count{
        unsafe{
            let page = alloc_page_phys_addr(1);
            let phys_addr = page.addr();
            if phys_addr == 0{
                panic!("Failed to alloc pages for kernel heap");
            }
            map_page_kernel(phys_addr, virt_addr, PTE_PRESENT | PTE_READ_WRITE);
        }
        virt_addr += page_size;
    }

    let base = heap_virtual_addr_start as *mut u8;
    let span = talc::Span::from_base_size(base, kernel_heap_size);
    unsafe{
        ALLOCATOR.lock().claim(span).expect("Failed to claim heap memory.");
    }

    println!("Heap allocator initialized.");
}