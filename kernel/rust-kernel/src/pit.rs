use crate::apic;

const CHANNEL_0: u8 = 0;
const ACCESS_LO_HI: u8 = 0b11 << 4;
const MODE_0: u8 = 0;
const BINARY_MODE: u8 = 0;
const COMMAND_PORT: u16 = 0x43;
const CHANNEL_0_PORT: u16 = 0x40;

pub static mut divisor: u16 = 1;

pub fn reload_pit(value: u16){
    let lo = (value & 0xff) as u8;
    let hi = ((value >> 8) & 0xff) as u8;
    let command = CHANNEL_0 | ACCESS_LO_HI | MODE_0 | BINARY_MODE;
    let mut command_port = x86_64::instructions::port::Port::new(COMMAND_PORT);
    let mut data_port = x86_64::instructions::port::Port::new(CHANNEL_0_PORT);
    unsafe{
        command_port.write(command);
        data_port.write(lo);
        data_port.write(hi);
    }
}

pub fn hz_to_pit_value(hz: u32) -> u16{
    (1193180 / hz) as u16
}

pub fn reload_pit_with_same_divisor(){
    reload_pit(unsafe { divisor });
}

pub fn setup_PIT(hz: u32){
    unsafe { divisor = hz_to_pit_value(hz) };
    reload_pit_with_same_divisor();
}

pub fn interrupt_apic(){
    update_sleep();
    apic::send_EOI();
    reload_pit_with_same_divisor();
}

static mut sleep_count: u64 = 0;
static mut sleeping: bool = false;

pub fn update_sleep(){
    unsafe{
        if sleeping && (sleep_count != 0){
            sleep_count -= 1;
        }
    }
}

pub fn prepare_sleep(ms: u64){
    unsafe{
        sleep_count = ms;
        if sleeping{
            panic!("Already sleeping");
        }
    }
}

pub fn perform_sleep(){
    setup_PIT(1000);
    unsafe{
        sleeping = true;
        while sleep_count != 0{
            x86_64::instructions::hlt();
        }
        sleeping = false;
    }
}