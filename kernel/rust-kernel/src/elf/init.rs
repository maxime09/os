use core::slice;

use elf::{endian::AnyEndian, segment::ProgramHeader, ElfBytes};

use crate::{alloc_page_phys_addr, map_page_current, PTE_PRESENT, PTE_READ_WRITE, PTE_USER_SUPERVISOR};

const PAGE_SIZE: usize = 4096;

pub fn load_init_elf(data: &[u8]) -> (usize, usize, usize, usize){
    let file = ElfBytes::<AnyEndian>::minimal_parse(data).unwrap();
    let segments = file.segments().unwrap();
    let mut end_of_segments = 0;
    for segment in segments{
        if segment.p_type == 1{
            let current_segment_end = load_segment(&segment, data);
            if current_segment_end > end_of_segments{
                end_of_segments = current_segment_end;
            }
        }
    }
    let stack_start = alloc_memory_zone(end_of_segments as usize, 2);
    let sp = stack_start + 2 * PAGE_SIZE - 16;
    let heap_len = 16 * PAGE_SIZE;
    let heap_addr = alloc_memory_zone(stack_start + 3 * PAGE_SIZE, heap_len / PAGE_SIZE);
    (file.ehdr.e_entry as usize, sp, heap_addr, heap_len)
}

pub fn load_segment(header: &ProgramHeader, data: &[u8]) -> usize{
    let alloc_size = header.p_memsz;
    let mut page_count = alloc_size / 4096;
    if (alloc_size % 4096) != 0{
        page_count += 1; // Add one page if alloc doesn't end on page boundary
    }
    let pages = unsafe { alloc_page_phys_addr(page_count.try_into().unwrap()) };
    for i in 0..page_count{
        let offset = (i * 4096) as usize;
        let phys_addr = (pages as usize) + offset;
        let virt_addr = (header.p_vaddr as usize) + offset;
        unsafe{
            map_page_current(phys_addr, virt_addr, PTE_PRESENT| PTE_READ_WRITE | PTE_USER_SUPERVISOR)
        }
    }

    let start_addr = header.p_vaddr as *mut u8;
    let slice = unsafe { slice::from_raw_parts_mut(start_addr, header.p_memsz as usize) };
    for i in 0..slice.len(){
        slice[i] = 0;
    }
    for i in 0..header.p_filesz as usize{
        let offset = (header.p_offset as usize) + i;
        slice[i] = data[offset];
    }
    let virt_addr = header.p_vaddr;
    let page_start = virt_addr - (virt_addr%4096);
    (page_start + 4096*page_count) as usize //return end of segment
}

pub fn alloc_memory_zone(base_addr: usize, page_count: usize) -> usize{
    let pages = unsafe {alloc_page_phys_addr(page_count)};
    for i in 0..page_count{
        let offset = (i * PAGE_SIZE) as usize;
        let phys_addr = (pages as usize) + offset;
        let virt_addr = (base_addr as usize) + offset;
        unsafe { map_page_current(phys_addr, virt_addr, PTE_PRESENT | PTE_READ_WRITE | PTE_USER_SUPERVISOR); }
    }
    base_addr
}