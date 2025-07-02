#![no_std]
#![no_main]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate alloc;

use core::{ffi::c_int, fmt, panic::PanicInfo, slice};
pub mod interrupts;
use alloc::{boxed::Box, vec::Vec};
use fs::vfs;
pub use interrupts::*;
use rsdt::MADT;
use spin::Mutex;
use x86_64::instructions::hlt;
pub mod fs;
//pub mod pci;
pub mod pit;
pub mod cpuid;
pub mod apic;
pub mod rsdt;
pub mod elf;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel panic: {}", info);
    loop {
        hlt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_kmain(initrd_ptr: *const core::ffi::c_void, initrd_size: usize, rsdp: *mut core::ffi::c_void) -> !{
    println!("Hello from rust!");
    
    let rsdt = unsafe { rsdt::RSDT::get_RSDT(rsdp) };
    let madt = MADT::from_rsdt(&rsdt);
    let io_apic_addr = unsafe {
        madt.get_ioapic_addr()
    };
    unsafe{
        apic::setup_io_apic_addr(io_apic_addr);
    }
    println!("I/O APIC version: {}", unsafe{apic::get_io_apic_version()});
    apic::set_task_priority(0);
    println!("reading initrd");

    let data = unsafe{Box::from_raw(slice::from_raw_parts_mut(initrd_ptr as *mut u8, initrd_size))};


    println!("parsing tar header");
    let headers = fs::ustar::parse_file(&data);

    let vfs = fs::ustar::headers_to_fs(headers, data);

    let mountpoint = vfs.get_mountpoint().unwrap();
   
    println!("reading init.elf");
    let init_elf_path = vfs::PathBuf::from("init.elf");
    let init_inode = vfs.find(init_elf_path).unwrap();
    let init_inode_size = init_inode.get_size(mountpoint).unwrap();
    let init_elf_data = vfs.read(mountpoint, init_inode, 0, init_inode_size).unwrap();
    
    println!("Setup apic");

    apic::setup_apic();
    println!("Setup pit");
    apic::setup_PIT_interrupt(&madt);
    println!("Setup keyboard");
    apic::setup_keyboard_interrupt(&madt);
    println!("Setup apic timer");
    apic::timer::setup_apic_timer();


    println!("Starting other cores if available");
    unsafe { start_slave_core() };


    let entry_point = elf::init::load_init_elf(&init_elf_data);
    println!("Entry point 0x{:x}", entry_point);

    unsafe{
        jump_to_usermode(entry_point);
    }

    println!("Execution ended");

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

#[global_allocator]
static ALLOCATOR: talc::Talck<spin::Mutex<()>, talc::ErrOnOom> = talc::Talc::new(talc::ErrOnOom).lock();

const kernel_heap_size: usize = 4*1024*1024; // 4 Mb
const heap_virtual_addr_start: usize = 0x1_000_000;
const page_size: usize = 4096;
const PTE_PRESENT: c_int = 1;
const PTE_READ_WRITE: c_int = 2;
const PTE_USER_SUPERVISOR: c_int = 4;

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

#[unsafe(no_mangle)]
pub extern "C" fn rust_slave_main(core_id: u32, rsdp: *mut core::ffi::c_void){
    let rsdt = unsafe { rsdt::RSDT::get_RSDT(rsdp) };
    let madt = MADT::from_rsdt(&rsdt);
    apic::setup_apic();

    println!("Core {} started", core_id);
}