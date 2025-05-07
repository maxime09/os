const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;

const PIC1_COMMAND: u16 = PIC1;
const PIC1_DATA: u16 = PIC1 + 1;
const PIC2_COMMAND: u16 = PIC2;
const PIC2_DATA: u16 = PIC2 + 1;

use crate::{inb, io_wait, outb};

#[unsafe(no_mangle)]
pub fn IRQ_set_mask(mut irq_num:  u8){
    let port = if irq_num < 8 {
        PIC1_DATA
    }else {
        irq_num -= 8;
        PIC2_DATA
    };
    unsafe{
        let value = inb(port) | (1 << irq_num);
        outb(port, value);
    }
}

#[unsafe(no_mangle)]
pub fn IRQ_clear_mask(mut irq_num:  u8){
    let port = if irq_num < 8 {
        PIC1_DATA
    }else {
        irq_num -= 8;
        PIC2_DATA
    };
    unsafe{
        let value = inb(port) & !(1 << irq_num);
        outb(port, value);
    }
}

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;

#[unsafe(no_mangle)]
pub fn PIC_remap(offset1: u8, offset2: u8){
    unsafe{
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_INIT); //start the initialization sequence
        io_wait();
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_INIT);
        io_wait();

        // set the offsets
        outb(PIC1_DATA, offset1);
        io_wait();
        outb(PIC2_DATA, offset2);
        io_wait();

        outb(PIC1_DATA, 4);
        io_wait();
        outb(PIC2_DATA, 2);
        io_wait();

        // Mask both PICs
        for i in 0..16{
            IRQ_set_mask(i);
            io_wait();
        } 
    }
}

const PIC_EOI: u8 = 0x20;

#[unsafe(no_mangle)]
pub fn PIC_sendEOI(irq: u8){
    unsafe {
        if irq >= 0x28 {
            outb(PIC2_COMMAND, PIC_EOI); // interrupts from the slave
        }
        // we need to send end of interrupt to the master in both case
        outb(PIC1_COMMAND, PIC_EOI);
    }
}