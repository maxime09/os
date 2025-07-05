use core::{alloc::Layout, ptr::NonNull};

use crate::usermode_switch;

pub struct Process{
    pid: usize,
    saved_stack_pointer: usize,
    saved_instruction_pointer: usize,
    process_allocator: talc::Talck<spin::Mutex<()>, talc::ErrOnOom>,
}

impl Process{
    pub fn new(pid: usize, ip: usize, sp: usize, heap_start: usize, heap_len: usize) -> Self{
        let mut allocator = talc::Talc::new(talc::ErrOnOom).lock();
        let span = talc::Span::from_base_size(heap_start as *mut u8, heap_len as usize);
        unsafe { allocator.lock().claim(span).expect("Failed to init process allocator") };
        Process { pid, saved_stack_pointer: sp, saved_instruction_pointer: ip, process_allocator: allocator }
    }


    pub unsafe fn execute_process(&self){
        unsafe{
            usermode_switch(self.saved_instruction_pointer, self.saved_stack_pointer);
        }
    }

    pub fn malloc(&self, layout: Layout) -> Result<NonNull<u8>, ()>{
        unsafe { self.process_allocator.lock().malloc(layout) }
    }
}