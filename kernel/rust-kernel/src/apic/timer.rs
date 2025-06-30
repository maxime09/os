use crate::{apic::{read_lapic, write_lapic}, pit};

const APIC_REGISTER_TIMER_DIV: u32 = 0x3E0;
const APIC_REGISTER_TIMER_INITCNT: u32 = 0x380;
const APIC_REGISTER_TIMER_CURRCNT: u32 = 0x390;
const APIC_REGISTER_LVT_TIMER: u32 = 0x320;
const APIC_LVT_INT_MASKED: u32 = (1<<16);
const APIC_TIMER_PERIODIC: u32 = 0x20000;

// Setup to apic timer to tick every 10 ms
pub fn setup_apic_timer(){
    // Set divider to 16
    write_lapic(APIC_REGISTER_TIMER_DIV, 0x3);

    // Sleep for 10ms using pit to know how many tick the apic timer did in 10ms
    pit::prepare_sleep(10);

    write_lapic(APIC_REGISTER_TIMER_INITCNT, 0xFFFF_FFFF);
    
    pit::perform_sleep();

    // Stop timer
    write_lapic(APIC_REGISTER_LVT_TIMER, APIC_LVT_INT_MASKED);

    let tickIn10ms = 0xFFFF_FFFF - read_lapic(APIC_REGISTER_TIMER_CURRCNT);

    // Start timer as periodic

    write_lapic(APIC_REGISTER_LVT_TIMER, 50 | APIC_TIMER_PERIODIC);
    write_lapic(APIC_REGISTER_TIMER_DIV, 0x3);
    write_lapic(APIC_REGISTER_TIMER_INITCNT, tickIn10ms);
}