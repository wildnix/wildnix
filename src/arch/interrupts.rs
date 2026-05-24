use crate::drv::serial::{inb, outb};

pub const PIC1_CMD: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_CMD: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

pub unsafe fn pic_remap() {
    unsafe {
        let a1 = inb(PIC1_DATA);
        let a2 = inb(PIC2_DATA);

        outb(PIC1_CMD, 0x11);
        outb(PIC2_CMD, 0x11);

        outb(PIC1_DATA, 0x20); // IRQ0-7  -> IDT 32-39
        outb(PIC2_DATA, 0x28); // IRQ8-15 -> IDT 40-47

        outb(PIC1_DATA, 4);
        outb(PIC2_DATA, 2);

        outb(PIC1_DATA, 0x01);
        outb(PIC2_DATA, 0x01);

        outb(PIC1_DATA, a1);
        outb(PIC2_DATA, a2);
    }
}

pub unsafe fn pic_set_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq = if irq < 8 { irq } else { irq - 8 };

    unsafe {
        let value = inb(port) | (1 << irq);
        outb(port, value);
    }
}

pub unsafe fn pic_clear_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq = if irq < 8 { irq } else { irq - 8 };

    unsafe {
        let value = inb(port) & !(1 << irq);
        outb(port, value);
    }
}

pub unsafe fn pic_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_CMD, PIC_EOI);
        }

        outb(PIC1_CMD, PIC_EOI);
    }
}
