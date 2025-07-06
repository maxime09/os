use core::{alloc::Layout, ptr::NonNull};

use alloc::collections::btree_map::BTreeMap;

use crate::usermode_switch;

pub struct Process{
    pid: usize,
    saved_stack_pointer: usize,
    saved_instruction_pointer: usize,
    allocations: spin::Mutex<ProcessAllocationData>
}

struct ProcessAllocationData{
    allocator: talc::Talc<talc::ErrOnOom>,
    allocations: BTreeMap<usize, Layout>,
}

impl ProcessAllocationData{
    fn new(heap_start: usize, heap_len: usize) -> Self{
        let mut allocator = talc::Talc::new(talc::ErrOnOom);
        let span = talc::Span::from_base_size(heap_start as *mut u8, heap_len);
        unsafe { allocator.claim(span) };
        let allocations = BTreeMap::new();
        ProcessAllocationData { allocator, allocations }
    }
}

impl Process{
    pub fn new(pid: usize, ip: usize, sp: usize, heap_start: usize, heap_len: usize) -> Self{
        let allocations = spin::Mutex::new(ProcessAllocationData::new(heap_start, heap_len));
        Process { pid, saved_stack_pointer: sp, saved_instruction_pointer: ip, allocations }
    }


    pub unsafe fn execute_process(&self){
        unsafe{
            usermode_switch(self.saved_instruction_pointer, self.saved_stack_pointer);
        }
    }

    pub fn malloc(&self, layout: Layout) -> usize{
        let mut allocations = self.allocations.lock();
        if let Ok(res) = unsafe { allocations.allocator.malloc(layout) }{
            let ptr = res.as_ptr().addr();
            allocations.allocations.insert(ptr, layout);
            ptr
        }else{
            0
        }
    }

    pub fn free(&self, ptr: u64){
        let mut allocations = self.allocations.lock();
        let addr = ptr as usize;
        let layout = allocations.allocations.remove(&addr).unwrap();
        let ptr = NonNull::new(ptr as *mut u8).unwrap();
        unsafe {
            allocations.allocator.free(ptr, layout);
        }

    }
}