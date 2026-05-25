use core::arch::asm;

const KERNEL_DATA: u16 = 0x10;
const TSS_SELECTOR: u16 = 0x28;

const GDT_KERNEL_CODE: u64 = 0x00AF9A000000FFFF;
const GDT_KERNEL_DATA: u64 = 0x00CF92000000FFFF;
const GDT_USER_DATA: u64 = 0x00CFF2000000FFFF;
const GDT_USER_CODE: u64 = 0x00AFFA000000FFFF;

#[repr(C, packed)]
struct Tss {
    _reserved0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    _reserved1: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    _reserved2: u64,
    _reserved3: u16,
    iopb_offset: u16,
}

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
    const fn user_code64() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0b11111010,      // present | ring3 | code | readable
            granularity: 0b10101111, // long mode
            base_high: 0,
        }
    }

    const fn user_data64() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0b11110010, // present | ring3 | data | writable
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

static mut GDT: [GdtEntry; 7] = [
    GdtEntry::null(),        // 0x00
    GdtEntry::code64(),      // 0x08 kernel code
    GdtEntry::data64(),      // 0x10 kernel data
    GdtEntry::user_data64(), // 0x18 | 3 = 0x1B
    GdtEntry::user_code64(), // 0x20 | 3 = 0x23
    GdtEntry::null(),        // 0x28 TSS low
    GdtEntry::null(),        // 0x30 TSS high
];

#[repr(align(16))]
struct AlignedStack([u8; 4096 * 4]);

static mut RING0_STACK: AlignedStack = AlignedStack([0; 4096 * 4]);

static mut TSS: Tss = Tss {
    _reserved0: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    _reserved1: 0,
    ist1: 0,
    ist2: 0,
    ist3: 0,
    ist4: 0,
    ist5: 0,
    ist6: 0,
    ist7: 0,
    _reserved2: 0,
    _reserved3: 0,
    iopb_offset: core::mem::size_of::<Tss>() as u16,
};

fn make_tss_descriptor(base: u64, limit: u32) -> (u64, u64) {
    let low = (limit as u64 & 0xFFFF)
        | ((base & 0x00FF_FFFF) << 16)
        | ((0x89u64) << 40)
        | (((limit as u64 >> 16) & 0xF) << 48)
        | (((base >> 24) & 0xFF) << 56);
    let high = (base >> 32) & 0xFFFF_FFFF;
    (low, high)
}

fn gdt_entry_from_u64(v: u64) -> GdtEntry {
    GdtEntry {
        limit_low: (v & 0xFFFF) as u16,
        base_low: ((v >> 16) & 0xFFFF) as u16,
        base_mid: ((v >> 32) & 0xFF) as u8,
        access: ((v >> 40) & 0xFF) as u8,
        granularity: ((v >> 48) & 0xFF) as u8,
        base_high: ((v >> 56) & 0xFF) as u8,
    }
}

pub fn init() {
    unsafe {
        GDT[1] = gdt_entry_from_u64(GDT_KERNEL_CODE);
        GDT[2] = gdt_entry_from_u64(GDT_KERNEL_DATA);
        GDT[3] = gdt_entry_from_u64(GDT_USER_DATA);
        GDT[4] = gdt_entry_from_u64(GDT_USER_CODE);

        TSS.rsp0 = (core::ptr::addr_of!(RING0_STACK.0) as u64) + core::mem::size_of::<AlignedStack>() as u64;
        let (tss_low, tss_high) = make_tss_descriptor(
            core::ptr::addr_of!(TSS) as u64,
            (core::mem::size_of::<Tss>() - 1) as u32,
        );
        GDT[5] = gdt_entry_from_u64(tss_low);
        GDT[6] = gdt_entry_from_u64(tss_high);

        let descriptor = GdtDescriptor {
            size: (core::mem::size_of::<[GdtEntry; 7]>() - 1) as u16,
            offset: core::ptr::addr_of!(GDT) as u64,
        };

        crate::drv::serial::write(b"gdt: loading descriptor\n");

        asm!("lgdt [{gdt}]", gdt = in(reg) &descriptor);

        reload_cs();

        asm!(
            "mov ax, {kernel_data}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "mov ax, {tss_sel}",
            "ltr ax",
            kernel_data = const KERNEL_DATA,
            tss_sel = const TSS_SELECTOR,
            out("ax") _,
        );

        crate::drv::serial::write(b"gdt: segments reloaded\n");
    }
}

#[unsafe(naked)]
unsafe extern "C" fn reload_cs() {
    core::arch::naked_asm!("pop rdi", "push 0x08", "push rdi", "retfq",);
}
