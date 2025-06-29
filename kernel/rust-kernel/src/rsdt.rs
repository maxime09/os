use core::slice;

use alloc::{string::String, vec::Vec};
use zerocopy::{Immutable, KnownLayout, TryFromBytes, Unaligned};

use crate::phys_addr_to_limine_virtual_addr;

#[derive(Debug, TryFromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct RSDP_t{
    signature: [u8; 8],
    checksum: u8,
    OEMID: [u8; 6],
    revision: u8,
    addr: u32
}

#[derive(Debug, TryFromBytes, KnownLayout, Immutable, Unaligned, Clone)]
#[repr(C, packed)]
pub struct ACPISTDHeader{
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    OEMID: [u8; 6],
    OEMTableID: [u8; 8],
    OEMRevision: u32,
    CreatorID: u32,
    CreatorRevision: u32
}

impl ACPISTDHeader{
    pub fn get_signature(&self) -> [u8;4]{
        self.signature
    }

    pub fn get_signature_as_str(&self) -> String{
        self.signature.iter().map(|c| *c as char).collect()
    }

    pub fn is_table(&self, table_signature: &[u8]) -> bool{
        self.signature == table_signature
    }
}

#[derive(Debug)]
pub struct RSDT{
    header: ACPISTDHeader,
    base_addr: *const u8,
    entry_count: usize,
}

impl RSDT{
    pub unsafe fn get_RSDT(rsdp: *mut core::ffi::c_void) -> Self{
        let rsdp_struct = unsafe{ parse_rsdp(rsdp) };
        let rsdt_addr = rsdp_struct.get_rsdt_addr();
        let header = get_ACPISTD_header(rsdt_addr);
        let entry_count = ((header.length as usize) - size_of::<ACPISTDHeader>()) / 4;
        RSDT{
            header,
            base_addr: rsdt_addr as *const u8,
            entry_count,
        }
    }

    pub unsafe fn len(&self) -> usize{
        self.entry_count
    }

    pub unsafe fn get_entries_addr(&self) -> Vec<*const core::ffi::c_void>{
        let mut res = Vec::with_capacity(self.entry_count);
        let mut start_addr = unsafe{self.base_addr.offset(size_of::<ACPISTDHeader>().try_into().unwrap())};
        for _ in 0..self.entry_count{
            let slice = unsafe {slice::from_raw_parts(start_addr, 4)};
            let addr = u32::try_read_from_bytes(slice).unwrap() as usize;
            let ptr = unsafe { phys_addr_to_limine_virtual_addr(addr) } as *const core::ffi::c_void;
            res.push(ptr);
            start_addr = unsafe{start_addr.offset(4)};
        }
        res
    }

    pub fn find_entry(&self, signature: &[u8]) -> Option<(*const core::ffi::c_void, ACPISTDHeader)>{
        let entries_addr = unsafe {
            self.get_entries_addr()
        };
        let entries = entries_addr.iter().map(|ptr| (*ptr, unsafe {
            get_ACPISTD_header(*ptr)
        })).collect::<Vec<_>>();
        entries.iter().filter(|(_, header)|{
            header.is_table(signature)
        }).next().cloned()
    }
}

impl RSDP_t{
    pub fn get_rsdt_addr(&self) -> *const core::ffi::c_void{
        if self.revision != 0{
            panic!("XSDT not supported yet");
        }
        (unsafe { phys_addr_to_limine_virtual_addr(self.addr as usize) }) as *const _
    }
}


pub unsafe fn parse_rsdp(rsdp: *mut core::ffi::c_void) -> RSDP_t{
    let slice = unsafe { slice::from_raw_parts(rsdp as *const u8, size_of::<RSDP_t>()) };
    RSDP_t::try_read_from_bytes(slice).unwrap()
}

pub unsafe fn get_ACPISTD_header(pointer: *const core::ffi::c_void) -> ACPISTDHeader{
    let slice = unsafe {
        slice::from_raw_parts(pointer as *const u8, size_of::<ACPISTDHeader>()) 
    };
    ACPISTDHeader::try_read_from_bytes(slice).unwrap()
}

#[derive(Debug)]
pub struct MADT{
    header: ACPISTDHeader,
    addr: *const u8,
}

#[derive(Debug)]
pub struct InterruptSourceOverride{
    bus: u8,
    irq: u8,
    GSI: u32,
    flags: u16,
}

impl MADT{
    pub fn from_ptr_and_header(ptr: *const core::ffi::c_void, header: ACPISTDHeader) -> Self{
        MADT { 
            header, 
            addr: ptr as *const u8
        }
    }

    pub fn from_rsdt(rsdt: &RSDT) -> Self{
        let (ptr, header) = rsdt.find_entry(b"APIC").expect("No MADT table found");
        Self::from_ptr_and_header(ptr, header)
    }

    pub unsafe fn get_entries_offset_with_type(&self, entry_type: u8) -> Vec<usize>{
        let mut result = Vec::new();
        let mut offset = size_of::<ACPISTDHeader>() + 8;
        while offset < self.header.length.try_into().unwrap(){
            let current_type = unsafe {self.addr.byte_offset(offset.try_into().unwrap()).read_unaligned()};
            let current_length = unsafe {self.addr.byte_offset((offset + 1).try_into().unwrap()).read_unaligned()};
            if current_type == entry_type{
                result.push(offset);
            }
            offset += current_length as usize
        }
        result
    }

    pub unsafe fn read_byte(&self, offset_of_entry: usize, offset_in_entry: usize) -> u8{
        unsafe{
            let addr = self.addr.byte_offset((offset_of_entry + offset_in_entry).try_into().unwrap()) as *const u8;
            addr.read_unaligned()
        }
    }

    pub unsafe fn read_u16(&self, offset_of_entry: usize, offset_in_entry: usize) -> u16{
        unsafe{
            let addr = self.addr.byte_offset((offset_of_entry + offset_in_entry).try_into().unwrap()) as *const u16;
            addr.read_unaligned()
        }
    }

    pub unsafe fn read_u32(&self, offset_of_entry: usize, offset_in_entry: usize) -> u32{
        unsafe{
            let addr = self.addr.byte_offset((offset_of_entry + offset_in_entry).try_into().unwrap()) as *const u32;
            addr.read_unaligned()
        }
    }

    pub unsafe fn get_ioapic_addr(&self) -> *const u8{
        unsafe {
            let entry_offset = self.get_entries_offset_with_type(1)
                .iter()
                .next()
                .cloned()
                .expect("No I/O APIC entry in MADT.");
            let addr = self.read_u32(entry_offset, 4) as usize;
            phys_addr_to_limine_virtual_addr(addr) as *const u8
        }
    }

    pub unsafe fn get_interrupt_overrides(&self) -> Vec<InterruptSourceOverride>{
        let mut result = Vec::new();
        unsafe{
            let entries_offset = self.get_entries_offset_with_type(2);
            for entry_offset in entries_offset{
                let bus = self.read_byte(entry_offset, 2);
                let irq = self.read_byte(entry_offset, 3);
                let GSI = self.read_u32(entry_offset, 4);
                let flags = self.read_u16(entry_offset, 8);
                result.push(InterruptSourceOverride { bus, irq, GSI, flags });
            }
        }
        result
    }

}