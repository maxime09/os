use core::alloc::Layout;

use crate::{keyboard, kputs, scheduler};

#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(
    rdi: u64,
    rsi: u64,
    rdx: u64,
    _rcx: u64,
    _r8: u64,
    _r9: u64,
) -> u64 {
    let mut rax = 0;
    match rdi {
        1 => {
            unsafe { kputs(rsi as *mut i8) };
        }
        2 => {
            syscall_exit(rsi);
        }
        3 => {
            syscall_read_keyboard(&mut rax);
        },
        4 => {
            syscall_alloc(rsi, rdx, &mut rax);
        },
        5 => {
            syscall_free(rsi);
        },
        6 => {
            file_syscall_handler();
        }
        _ => {
            panic!("Unknown syscall {}", rdi);
        }
    }
    rax
}

pub fn syscall_exit(_exit_code: u64) {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        scheduler
            .get()
            .as_mut()
            .unwrap()
            .assume_init_mut()
            .end_current_process();
    });
}

pub fn syscall_read_keyboard(rax: &mut u64) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(input) = keyboard::pop_input() {
            *rax = input as u64;
        } else {
            *rax = 0;
        }
    });
}

pub fn syscall_alloc(size: u64, align: u64, out: &mut u64){
    x86_64::instructions::interrupts::without_interrupts(|| {
        let layout = Layout::from_size_align(size as usize, align as usize).unwrap();
        let process = unsafe { scheduler.get().as_mut().unwrap().assume_init_mut()
            .get_current_process()
            .unwrap()
        };
        *out = process.malloc(layout) as u64;
    })
}

pub fn syscall_free(ptr: u64){
    x86_64::instructions::interrupts::without_interrupts(||{
        let process = unsafe {
            scheduler.get().as_mut().unwrap().assume_init_mut()
            .get_current_process()
            .unwrap()
        };
        process.free(ptr);
    })
}

pub fn file_syscall_handler(){

}