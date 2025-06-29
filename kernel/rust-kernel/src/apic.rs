use x86_64::registers::model_specific::Msr;

use crate::{alloc_page_phys_addr, map_page_kernel, PTE_PRESENT, PTE_READ_WRITE};

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_BSP: u64= 0x100;
const IA32_APIC_BASE_MSR_ENABLE: u64= 0x800;

static mut base_addr: u64 = 0;

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
        alloc_mem();
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
    }
}

pub fn read_io_apic(ioapicaddr: *const u8, reg: u32) -> u32{
    unsafe{
        let address_register_ptr = ioapicaddr.byte_offset(0) as *mut u32;
        let data_register_ptr = ioapicaddr.byte_offset(0x10) as *const u32;
    
        address_register_ptr.write_volatile(reg & 0xff);
        data_register_ptr.read_volatile()
    }
}

pub fn write_io_apic(ioapicaddr: *const u8, reg: u32, value: u32){
    unsafe{
        let address_register_ptr = ioapicaddr.byte_offset(0) as *mut u32;
        let data_register_ptr = ioapicaddr.byte_offset(0x10) as *mut u32;
    
        address_register_ptr.write_volatile(reg & 0xff);
        data_register_ptr.write_volatile(value);
    }
}

pub fn setup_interrupt_redirection(ioapicaddr: *const u8, redirection_index: u32, interrupt_vector: u8, delivery_mode: u8, logical_destination: bool, invert_polarity: bool, level_trigger: bool, mask: bool, destination: u8){
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

    write_io_apic(ioapicaddr, 0x10 + (2 * redirection_index), low);
    write_io_apic(ioapicaddr, 0x10 + (2 * redirection_index) + 1, high);
}

pub fn setup_keyboard_interrupt(ioapicaddr: *const u8){
    setup_interrupt_redirection(ioapicaddr,
        1, 
        0x40, 
        0x00, 
        false, 
        false, 
        false, 
        false, 
        0);
}