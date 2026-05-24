use core::arch::asm;

use crate::mem::pmm;

const PAGE_SIZE: u64 = 4096;

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const HUGE: u64 = 1 << 7;

const USER: u64 = 1 << 2;

pub const FLAG_USER: u64 = USER;

static mut HHDM_OFFSET: u64 = 0;

#[repr(transparent)]
pub struct PageTable {
    entries: [u64; 512],
}

fn phys_to_virt(phys: u64) -> *mut PageTable {
    unsafe { (phys + HHDM_OFFSET) as *mut PageTable }
}

fn read_cr3() -> u64 {
    let value: u64;

    unsafe {
        asm!("mov {}, cr3", out(reg) value);
    }

    value & 0x000f_ffff_ffff_f000
}

pub unsafe fn init(hhdm_offset: u64) {
    unsafe {
        HHDM_OFFSET = hhdm_offset;
    }

    crate::drv::serial::write(b"vmm: initialized\n");
}

pub unsafe fn map_page(virt: u64, phys: u64, flags: u64) {
    let pml4_phys = read_cr3();
    let pml4 = phys_to_virt(pml4_phys);

    let pml4_i = ((virt >> 39) & 0x1ff) as usize;
    let pdpt_i = ((virt >> 30) & 0x1ff) as usize;
    let pd_i = ((virt >> 21) & 0x1ff) as usize;
    let pt_i = ((virt >> 12) & 0x1ff) as usize;

    unsafe {
        let pdpt = get_or_create_table(&mut (*pml4).entries[pml4_i]);
        let pd = get_or_create_table(&mut (*pdpt).entries[pdpt_i]);
        let pt = get_or_create_table(&mut (*pd).entries[pd_i]);

        (*pt).entries[pt_i] = (phys & 0x000f_ffff_ffff_f000) | flags | PRESENT;

        invlpg(virt);
    }
}

unsafe fn get_or_create_table(entry: &mut u64) -> *mut PageTable {
    if (*entry & PRESENT) == 0 {
        let phys = pmm::alloc().expect("pmm alloc failed for page table");

        let table = phys_to_virt(phys);

        unsafe {
            for i in 0..512 {
                (*table).entries[i] = 0;
            }
        }

        *entry = phys | PRESENT | WRITABLE | USER;
    }

    phys_to_virt(*entry & 0x000f_ffff_ffff_f000)
}

unsafe fn invlpg(addr: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

pub unsafe fn map_range(virt_start: u64, phys_start: u64, size: u64, flags: u64) {
    let pages = size.div_ceil(PAGE_SIZE);

    for i in 0..pages {
        unsafe {
            map_page(
                virt_start + i * PAGE_SIZE,
                phys_start + i * PAGE_SIZE,
                flags,
            );
        }
    }
}

pub const FLAG_PRESENT: u64 = PRESENT;
pub const FLAG_WRITABLE: u64 = WRITABLE;
pub const FLAG_HUGE: u64 = HUGE;
