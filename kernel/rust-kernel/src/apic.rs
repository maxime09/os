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
}