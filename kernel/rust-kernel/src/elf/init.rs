use core::slice;

use elf::{endian::AnyEndian, segment::ProgramHeader, ElfBytes};

use crate::{alloc_page_phys_addr, map_page_current, println, PTE_PRESENT, PTE_READ_WRITE, PTE_USER_SUPERVISOR};


pub fn load_init_elf(data: &[u8]) -> usize{
    let file = ElfBytes::<AnyEndian>::minimal_parse(data).unwrap();
    let segments = file.segments().unwrap();
    for segment in segments{
        if segment.p_type == 1{
            load_segment(&segment, data);
        }
    }
    file.ehdr.e_entry as usize
}

pub fn load_segment(header: &ProgramHeader, data: &[u8]){
    let alloc_size = header.p_memsz;
    let mut page_count = alloc_size / 4096;
    if (alloc_size % 4096) != 0{
        page_count += 1; // Add one page if alloc doesn't end on page boundary
    }
    let pages = unsafe { alloc_page_phys_addr(page_count.try_into().unwrap()) };
    for i in 0..page_count{
        let offset = (i * page_count) as usize;
        let phys_addr = (pages as usize) + offset;
        let virt_addr = (header.p_vaddr as usize) + offset;
        unsafe{
            map_page_current(phys_addr, virt_addr, PTE_PRESENT | PTE_READ_WRITE | PTE_USER_SUPERVISOR)
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
}