use core::arch::asm;

#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    attributes: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            attributes: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn new(handler: u64, selector: u16, attributes: u8) -> Self {
        Self {
            offset_low: (handler & 0xFFFF) as u16,
            selector,
            ist: 0,
            attributes,
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFF_FFFF) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, align(16))]
struct IdtTable([IdtEntry; 256]);

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

const INTERRUPT_GATE: u8 = 0b1000_1110;

static mut IDT: IdtTable = IdtTable([IdtEntry::missing(); 256]);
static mut IDT_DESCRIPTOR: IdtDescriptor = IdtDescriptor { size: 0, offset: 0 };

pub fn init() {
    unsafe {
        crate::drv::serial::write(b"idt: setting handlers\n");

        IDT.0[0] = IdtEntry::new(handler_de as u64, 0x08, INTERRUPT_GATE);
        IDT.0[6] = IdtEntry::new(handler_ud as u64, 0x08, INTERRUPT_GATE);
        IDT.0[8] = IdtEntry::new(handler_df as u64, 0x08, INTERRUPT_GATE);
        IDT.0[13] = IdtEntry::new(handler_gp as u64, 0x08, INTERRUPT_GATE);
        IDT.0[14] = IdtEntry::new(handler_pf as u64, 0x08, INTERRUPT_GATE);

        for i in 32..48usize {
            IDT.0[i] = IdtEntry::new(handler_spurious as u64, 0x08, INTERRUPT_GATE);
        }

        IDT.0[33] = IdtEntry::new(handler_irq1_keyboard as u64, 0x08, INTERRUPT_GATE);

        let idt_addr = (&raw const IDT.0) as u64;
        crate::drv::serial::write(b"idt addr = ");
        crate::drv::serial::write_hex(idt_addr);
        crate::drv::serial::write(b"\n");

        IDT_DESCRIPTOR = IdtDescriptor {
            size: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            offset: idt_addr,
        };

        crate::drv::serial::write(b"idt: loading descriptor\n");

        asm!("lidt [{0}]", in(reg) &IDT_DESCRIPTOR, options(nostack));

        crate::drv::serial::write(b"idt: lidt done\n");
    }
}

macro_rules! exception_handler {
    ($name:ident, $msg:literal) => {
        extern "x86-interrupt" fn $name(_frame: InterruptStackFrame) {
            unsafe {
                crate::drv::serial::write(b"EXCEPTION: ");
                crate::drv::serial::write($msg);
                crate::drv::serial::write(b"\n");
            }
            loop {
                unsafe {
                    asm!("hlt");
                }
            }
        }
    };
}

macro_rules! exception_handler_err {
    ($name:ident, $msg:literal) => {
        extern "x86-interrupt" fn $name(_frame: InterruptStackFrame, _error: u64) {
            unsafe {
                crate::drv::serial::write(b"EXCEPTION: ");
                crate::drv::serial::write($msg);
                crate::drv::serial::write(b"\n");
            }
            loop {
                unsafe {
                    asm!("hlt");
                }
            }
        }
    };
}

extern "x86-interrupt" fn handler_pf(_frame: InterruptStackFrame, error: u64) {
    unsafe {
        let cr2: u64;
        core::arch::asm!("mov {}, cr2", out(reg) cr2);

        for &b in b"#PF CR2=" {
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") b);
        }
        for i in (0..16).rev() {
            let nibble = ((cr2 >> (i * 4)) & 0xF) as u8;
            let c = if nibble < 10 {
                b'0' + nibble
            } else {
                b'a' + nibble - 10
            };
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") c);
        }
        for &b in b" ERR=" {
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") b);
        }
        for i in (0..16).rev() {
            let nibble = ((error >> (i * 4)) & 0xF) as u8;
            let c = if nibble < 10 {
                b'0' + nibble
            } else {
                b'a' + nibble - 10
            };
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") c);
        }
        for &b in b"\nRIP=" {
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") b);
        }
        let rip = _frame.rip;
        for i in (0..16).rev() {
            let nibble = ((rip >> (i * 4)) & 0xF) as u8;
            let c = if nibble < 10 {
                b'0' + nibble
            } else {
                b'a' + nibble - 10
            };
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") c);
        }
        for &b in b"\n" {
            core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") b);
        }
    }
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

extern "x86-interrupt" fn handler_spurious(_frame: InterruptStackFrame) {
    unsafe {
        crate::arch::interrupts::pic_eoi(0);
    }
}

extern "x86-interrupt" fn handler_irq1_keyboard(_frame: InterruptStackFrame) {
    unsafe {
        crate::drv::keyboard::handle_irq();
        crate::arch::interrupts::pic_eoi(1);
    }
}

exception_handler!(handler_de, b"#DE divide by zero");
exception_handler!(handler_ud, b"#UD invalid opcode");
exception_handler_err!(handler_df, b"#DF double fault");
exception_handler_err!(handler_gp, b"#GP general protection");
