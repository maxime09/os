#![no_std]
#![no_main]

#![feature(sync_unsafe_cell)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate alloc;

use core::{cell::SyncUnsafeCell, ffi::c_int, mem::MaybeUninit, panic::PanicInfo, slice};
pub mod interrupts;
use alloc::boxed::Box;
use fs::vfs;
pub use interrupts::*;
use rsdt::MADT;
use x86_64::instructions::hlt;

use crate::scheduler::{process::Process, Scheduler};

pub mod fs;
pub mod pci;
pub mod pit;
pub mod cpuid;
pub mod apic;
pub mod rsdt;
pub mod elf;
pub mod scheduler;
pub mod allocator;
pub mod print;



const PTE_PRESENT: c_int = 1;
const PTE_READ_WRITE: c_int = 2;
const PTE_USER_SUPERVISOR: c_int = 4;

static scheduler: SyncUnsafeCell<MaybeUninit<Scheduler>> = SyncUnsafeCell::new(MaybeUninit::uninit());

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

    println!("Setup apic");

    apic::setup_apic();
    
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
    
    
    println!("Setup pit");
    apic::setup_PIT_interrupt(&madt);
    println!("Setup keyboard");
    apic::setup_keyboard_interrupt(&madt);
    println!("Setup apic timer");
    apic::timer::setup_apic_timer();


    println!("Starting other cores if available");
    unsafe { start_slave_core() };


    let (entry_point, sp, heap_start, heap_len) = elf::load_elf_file(&init_elf_data);
    println!("Entry point 0x{:x}", entry_point);
    unsafe { scheduler.get().as_mut().unwrap().write(Scheduler::new())};
    let init_process = Process::new(1, entry_point, sp, heap_start, heap_len);
    

    unsafe{
        let scheduler_ref = scheduler.get().as_mut().unwrap().assume_init_mut();
        scheduler_ref.add_to_queue(init_process);
        scheduler_ref.next_process();
        scheduler_ref.resume_current_process();
    }

    println!("Execution ended");

    loop {
        hlt();
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn rust_slave_main(_core_id: u32, rsdp: *mut core::ffi::c_void){
    let rsdt = unsafe { rsdt::RSDT::get_RSDT(rsdp) };
    let _madt = MADT::from_rsdt(&rsdt);
    apic::setup_apic();
    x86_64::instructions::interrupts::enable();
    loop {
        hlt();
    }
}