use core::arch::asm;

use alloc::string::String;

use crate::println;

pub fn get_vendor_id(){
    let mut result: [u32; 3] = [0; 3];
    let mut word1;
    let mut word2;
    let mut word3;
    let mut max_value: u32;
    unsafe{
        asm!(
            "push rbx",
            "mov eax, 0",
            "cpuid",
            "mov {tmp:e}, ebx",
            "pop rbx",
            tmp = out(reg) word1,
            out("edx") word2,
            out("ecx") word3,
            out("eax") max_value,
        );
    }
    result[0] = word1;
    result[1] = word2;
    result[2] = word3;
    let result_u8 = unsafe{
        core::mem::transmute::<[u32; 3], [u8; 12]>(result)
    };
    let result_string = result_u8.iter().map(|x| *x as char).collect::<String>();
    println!("{:?} max_cpuid_value: {}", result_string, max_value);
    if result_string != "GenuineIntel"{
        panic!("Only intel cpu are supported yet.")
    }
}

pub fn cpuid_01h() -> (u32, u32, u32, u32){
    let mut eax;
    let mut ebx;
    let mut ecx;
    let mut edx;
    unsafe {
        asm!(
            "push rbx",
            "mov eax, 1",
            "cpuid",
            "mov {tmp:e}, ebx",
            "pop rbx",
            tmp = out(reg) ebx,
            out("eax") eax,
            out("ecx") ecx,
            out("edx") edx
        );
    }
    (eax, ebx, ecx, edx)
}