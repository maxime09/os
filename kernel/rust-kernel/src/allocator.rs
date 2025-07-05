//const kernel_heap_size: usize = 4*1024*1024; // 4 Mb
//const heap_virtual_addr_start: usize = 0x1_000_000;
const page_size: usize = 4096;

use crate::{PTE_PRESENT, PTE_READ_WRITE, map_page_kernel, println, alloc_page_phys_addr};


#[global_allocator]
static ALLOCATOR: talc::Talck<spin::Mutex<()>, talc::ErrOnOom> = talc::Talc::new(talc::ErrOnOom).lock();

#[unsafe(no_mangle)]
pub extern "C" fn init_alloc(heap_virtual_addr_start: usize, heap_end: usize){
    let kernel_heap_size= heap_end - heap_virtual_addr_start;
    println!("Kernel heap size: 0x{:x}", kernel_heap_size);
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