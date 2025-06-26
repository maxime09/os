const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub fn ConfigReadWord(bus: u8, slot: u8, func: u8, offset: u8){
    let address = ((bus as u32) << 16) | ((slot as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xFC) | 0x80000000;
    unsafe {
        
    }
}