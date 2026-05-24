use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use crate::mem::{pmm, vmm};

const HEAP_START: u64 = 0xFFFF_A000_0000_0000;
const HEAP_SIZE: u64 = 1024 * 1024; // 1 MiB
const PAGE_SIZE: u64 = 4096;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

static mut HEAP_NEXT: u64 = HEAP_START;
static mut HEAP_END: u64 = HEAP_START + HEAP_SIZE;

pub unsafe fn init() {
    crate::drv::serial::write(b"heap: mapping\n");

    let pages = HEAP_SIZE / PAGE_SIZE;

    for i in 0..pages {
        let phys = unsafe { pmm::alloc().expect("heap pmm alloc failed") };

        unsafe {
            vmm::map_page(
                HEAP_START + i * PAGE_SIZE,
                phys,
                vmm::FLAG_PRESENT | vmm::FLAG_WRITABLE,
            );
        }
    }

    crate::drv::serial::write(b"heap: initialized\n");
}

pub struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align() as u64;
        let size = layout.size() as u64;

        unsafe {
            let start = align_up(HEAP_NEXT, align);
            let end = start + size;

            if end > HEAP_END {
                return null_mut();
            }

            HEAP_NEXT = end;

            start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump allocator does not free yet
    }
}

fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}
