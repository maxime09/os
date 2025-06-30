use x86_64::registers::model_specific::Msr;

use crate::{alloc_page_phys_addr, map_page_kernel, rsdt::MADT, PTE_PRESENT, PTE_READ_WRITE};
pub mod timer;


const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_BSP: u64= 0x100;
const IA32_APIC_BASE_MSR_ENABLE: u64= 0x800;

static mut base_addr: u64 = 0;
static mut io_apic_addr: *const u8 = core::ptr::null();

pub fn has_local_apic() -> bool{
    let (_, _, _, edx) = crate::cpuid::cpuid_01h();
    (edx & (1 << 9)) != 0
}

pub fn local_apic_id() -> u8{
    let (_, ebx, _, _) = crate::cpuid::cpuid_01h();
    ((ebx >> 24) & 0xff) as u8
}

pub fn get_APIC_BASE() -> u64{
    let msr = Msr::new(IA32_APIC_BASE_MSR);
    unsafe {
        msr.read() & 0xfffffffffffff000
    }
}

pub fn is_bsp() -> bool{
    let msr = Msr::new(IA32_APIC_BASE_MSR);
    unsafe {
        (msr.read() & IA32_APIC_BASE_MSR_BSP) != 0
    }
}

pub unsafe fn set_APIC_BASE(phys_addr: u64, is_BSP: bool){
    let mut msr = Msr::new(IA32_APIC_BASE_MSR);
    let mut value = phys_addr | IA32_APIC_BASE_MSR_ENABLE;
    if is_BSP{
        value |= IA32_APIC_BASE_MSR_BSP;
    }
    unsafe{
        msr.write(value);
    }
}

pub unsafe fn alloc_mem(){
    if unsafe { base_addr } == 0{
        let page = unsafe { alloc_page_phys_addr(1) };
        let addr = page.addr();
        unsafe { 
            map_page_kernel(addr, addr, PTE_PRESENT | PTE_READ_WRITE);
            base_addr = addr as u64;
        }
    }
}

pub fn setup_apic(){
    crate::pic::PIC_remap(0x20, 0x28); // Remap the PIC and mask all its interrupts to deactivate it
    if !has_local_apic(){
        panic!("APIC not supported");
    }
    unsafe { 
        base_addr  = get_APIC_BASE();
        map_page_kernel(base_addr as usize, base_addr as usize, PTE_PRESENT | PTE_READ_WRITE);
        set_APIC_BASE(base_addr, is_bsp());
    };
    if unsafe { base_addr } != get_APIC_BASE(){
        panic!("Failed to setup APIC base address");
    }

    write_lapic(0xF0, read_lapic(0xF0) | 0x100);
}

pub fn read_lapic(reg: u32) -> u32{
    unsafe{
        (base_addr as *const u32)
            .byte_offset(reg.try_into().unwrap())
            .read_volatile()
    }
}

pub fn write_lapic(reg: u32, value: u32){
    unsafe{
        (base_addr as *mut u32)
            .byte_offset(reg.try_into().unwrap())
            .write_volatile(value);
        (base_addr as *mut u32)
            .byte_offset(0x20)
            .read_volatile(); // read to ensure we wait long enough for the write to complete
    }
}

pub unsafe fn setup_io_apic_addr(ioapicaddr: *const u8){
    unsafe{
        io_apic_addr = ioapicaddr;
    }
}

pub unsafe fn read_io_apic(reg: u32) -> u32{
    unsafe{
        let address_register_ptr = io_apic_addr.byte_offset(0) as *mut u32;
        let data_register_ptr = io_apic_addr.byte_offset(0x10) as *const u32;
    
        address_register_ptr.write_volatile(reg & 0xff);
        data_register_ptr.read_volatile()
    }
}

pub unsafe fn write_io_apic(reg: u32, value: u32){
    unsafe{
        let address_register_ptr = io_apic_addr.byte_offset(0) as *mut u32;
        let data_register_ptr = io_apic_addr.byte_offset(0x10) as *mut u32;
    
        address_register_ptr.write_volatile(reg & 0xff);
        data_register_ptr.write_volatile(value);
    }
}


pub fn setup_interrupt_redirection(redirection_index: u32, interrupt_vector: u8, delivery_mode: u8, logical_destination: bool, invert_polarity: bool, level_trigger: bool, mask: bool, destination: u8){
    // TODO read previous value to keep reserved bits value
    
    let mut low = interrupt_vector as u32;
    low |= ((delivery_mode & 0x0F) as u32) << 8;
    if logical_destination{
        low |= 1 << 11;
    }
    if invert_polarity{
        low |= 1 << 13;
    }
    if level_trigger{
        low |= 1 << 15;
    }
    if mask {
        low |= 1 << 16;
    }

    let high = (destination as u32) << 24;

    unsafe{
        write_io_apic(0x10 + (2 * redirection_index), low);
        write_io_apic(0x10 + (2 * redirection_index) + 1, high);
    }
}

pub fn send_EOI(){
    write_lapic(0xB0, 0);
}

pub fn setup_keyboard_interrupt(madt: &MADT){
    let index = madt.find_override(1).unwrap_or(1);
    setup_interrupt_redirection(
        index, 
        0x31, 
        0x00, 
        false, 
        false, 
        false, 
        false, 
        0);
}

pub fn setup_PIT_interrupt(madt: &MADT){
    let index = madt.find_override(0).unwrap_or(0);
    setup_interrupt_redirection(
        index, 
        0x30, 
        0x00, 
        false, 
        false, 
        false, 
        false, 
        0);
}

pub unsafe fn get_io_apic_version() -> u8{
    unsafe{
        (read_io_apic(1) & 0xff) as u8
    }
}

pub fn set_task_priority(priority: u8){
    let previous_value = read_lapic(0x80) & 0x0f ;
    let new_value = ((priority as u32 & 0x0f) << 4) | previous_value;
    write_lapic(0x80, new_value);
}