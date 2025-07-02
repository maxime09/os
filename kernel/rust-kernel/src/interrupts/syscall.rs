use crate::println;

#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(rdi: u64, rsi: u64, rdx: u64, rcx: u64, r8: u64, r9: u64){
    println!("Received syscall");
}