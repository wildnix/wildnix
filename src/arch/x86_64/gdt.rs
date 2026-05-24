use core::arch::asm;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    const fn code64() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0b10011010,
            granularity: 0b10101111,
            base_high: 0,
        }
    }

    const fn data64() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0b10010010,
            granularity: 0b11001111,
            base_high: 0,
        }
    }
}

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

static GDT: [GdtEntry; 3] = [GdtEntry::null(), GdtEntry::code64(), GdtEntry::data64()];

static mut GDT_DESCRIPTOR: GdtDescriptor = GdtDescriptor { size: 0, offset: 0 };

pub fn init() {
    unsafe {
        GDT_DESCRIPTOR = GdtDescriptor {
            size: (core::mem::size_of::<[GdtEntry; 3]>() - 1) as u16,
            offset: GDT.as_ptr() as u64,
        };

        crate::drv::serial::write(b"gdt: loading descriptor\n");

        asm!(
            "lgdt [{0}]",
            in(reg) &GDT_DESCRIPTOR,
            options(nostack)
        );

        crate::drv::serial::write(b"gdt: lgdt done\n");
        reload_cs();
        crate::drv::serial::write(b"gdt: segments reloaded\n");
    }
}

#[unsafe(naked)]
unsafe extern "C" fn reload_cs() {
    core::arch::naked_asm!(
        "pop rdi",   // save return address
        "push 0x08", // CS
        "push rdi",  // return address
        "retfq",
    );
}
