use core::arch::asm;

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
            offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

// attributes for a present, ring0 interrupt gate
const INTERRUPT_GATE: u8 = 0b10001110;

static mut IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];

pub fn init() {
    unsafe {
        crate::drv::serial::write(b"idt: setting handlers\n");

        IDT[0] = IdtEntry::new(handler_de as u64, 0x08, INTERRUPT_GATE);
        IDT[6] = IdtEntry::new(handler_ud as u64, 0x08, INTERRUPT_GATE);
        IDT[8] = IdtEntry::new(handler_df as u64, 0x08, INTERRUPT_GATE);
        IDT[13] = IdtEntry::new(handler_gp as u64, 0x08, INTERRUPT_GATE);
        IDT[14] = IdtEntry::new(handler_pf as u64, 0x08, INTERRUPT_GATE);

        crate::drv::serial::write(b"idt: loading descriptor\n");

        let descriptor = IdtDescriptor {
            size: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            offset: IDT.as_ptr() as u64,
        };

        asm!("lidt [{0}]", in(reg) &descriptor, options(nostack));

        crate::drv::serial::write(b"idt: lidt done\n");

        asm!("sti");

        crate::drv::serial::write(b"idt: interrupts enabled\n");
    }
}

#[repr(C)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

macro_rules! exception_handler {
    ($name:ident, $msg:literal) => {
        extern "x86-interrupt" fn $name(frame: InterruptFrame) {
            unsafe {
                crate::drv::serial::write(b"EXCEPTION: ");
                crate::drv::serial::write($msg);
                crate::drv::serial::write(b"\n");
            }
            loop {
                unsafe { asm!("hlt") };
            }
        }
    };
}

macro_rules! exception_handler_err {
    ($name:ident, $msg:literal) => {
        extern "x86-interrupt" fn $name(frame: InterruptFrame, _error: u64) {
            unsafe {
                crate::drv::serial::write(b"EXCEPTION: ");
                crate::drv::serial::write($msg);
                crate::drv::serial::write(b"\n");
            }
            loop {
                unsafe { asm!("hlt") };
            }
        }
    };
}

exception_handler!(handler_de, b"#DE divide by zero");
exception_handler!(handler_ud, b"#UD invalid opcode");
exception_handler_err!(handler_df, b"#DF double fault");
exception_handler_err!(handler_gp, b"#GP general protection");
exception_handler_err!(handler_pf, b"#PF page fault");
