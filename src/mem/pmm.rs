use limine::memmap::MEMMAP_USABLE;

const PAGE_SIZE: usize = 4096;

fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

struct Bitmap {
    data: *mut u8,
    size: usize,
}

impl Bitmap {
    unsafe fn new(addr: *mut u8, size_bits: usize) -> Self {
        let byte_size = size_bits.div_ceil(8);

        let mut i = 0;

        while i + 8 <= byte_size {
            unsafe {
                (addr.add(i) as *mut u64).write_volatile(u64::MAX);
            }
            i += 8;
        }

        while i < byte_size {
            unsafe {
                addr.add(i).write_volatile(0xFF);
            }
            i += 1;
        }

        Self {
            data: addr,
            size: size_bits,
        }
    }

    unsafe fn set(&mut self, bit: usize, value: bool) {
        if bit >= self.size {
            return;
        }

        let byte = bit / 8;
        let mask = 1u8 << (bit % 8);

        unsafe {
            let ptr = self.data.add(byte);

            if value {
                *ptr |= mask;
            } else {
                *ptr &= !mask;
            }
        }
    }

    unsafe fn get(&self, bit: usize) -> bool {
        if bit >= self.size {
            return true;
        }

        let byte = bit / 8;
        let mask = 1u8 << (bit % 8);

        unsafe { (*self.data.add(byte) & mask) != 0 }
    }

    unsafe fn find_free(&self) -> Option<usize> {
        for i in 0..self.size {
            if unsafe { !self.get(i) } {
                return Some(i);
            }
        }

        None
    }
}

static mut BITMAP: Option<Bitmap> = None;
static mut TOTAL_PAGES: usize = 0;
static mut FREE_PAGES: usize = 0;

pub unsafe fn init(hhdm_offset: u64) {
    crate::drv::serial::write(b"pmm: init entered\n");

    let response = match crate::MEMORY_MAP_REQUEST.response() {
        Some(response) => {
            crate::drv::serial::write(b"pmm: memmap response ok\n");
            response
        }
        None => {
            crate::drv::serial::write(b"pmm: memmap response NONE\n");
            panic!("no memory map response");
        }
    };

    let entries = response.entries();
    crate::drv::serial::write(b"pmm: entries ok\n");

    let mut highest_addr: u64 = 0;

    for entry in entries {
        let end = entry.base + entry.length;

        if end > highest_addr {
            highest_addr = end;
        }
    }

    let total_pages = (highest_addr as usize).div_ceil(PAGE_SIZE);
    let bitmap_size = total_pages.div_ceil(8);

    crate::drv::serial::write(b"pmm: calculated sizes\n");

    let mut bitmap_phys: u64 = 0;

    for entry in entries {
        if entry.type_ != MEMMAP_USABLE {
            continue;
        }

        let start = align_up(entry.base.max(0x1000), PAGE_SIZE as u64);
        let end = entry.base + entry.length;

        if end > start && (end - start) as usize >= bitmap_size {
            bitmap_phys = start;
            break;
        }
    }

    if bitmap_phys == 0 {
        crate::drv::serial::write(b"pmm: no memory for bitmap\n");
        panic!("no memory for bitmap");
    }

    let bitmap_addr = (bitmap_phys + hhdm_offset) as *mut u8;

    crate::drv::serial::write(b"pmm: bitmap_phys = ");
    crate::drv::serial::write_hex(bitmap_phys);
    crate::drv::serial::write(b"\n");

    crate::drv::serial::write(b"pmm: hhdm_offset = ");
    crate::drv::serial::write_hex(hhdm_offset);
    crate::drv::serial::write(b"\n");

    crate::drv::serial::write(b"pmm: bitmap_addr = ");
    crate::drv::serial::write_hex(bitmap_addr as u64);
    crate::drv::serial::write(b"\n");

    crate::drv::serial::write(b"pmm: before bitmap new\n");

    let mut bitmap = unsafe { Bitmap::new(bitmap_addr, total_pages) };

    crate::drv::serial::write(b"pmm: bitmap created\n");

    let mut free_pages: usize = 0;

    for entry in entries {
        if entry.type_ != MEMMAP_USABLE {
            continue;
        }

        let start = align_up(entry.base.max(0x1000), PAGE_SIZE as u64);
        let end = entry.base + entry.length;

        if end <= start {
            continue;
        }

        let start_page = start as usize / PAGE_SIZE;
        let page_count = ((end - start) as usize) / PAGE_SIZE;

        for i in 0..page_count {
            unsafe {
                bitmap.set(start_page + i, false);
            }

            free_pages += 1;
        }
    }

    crate::drv::serial::write(b"pmm: marked usable free\n");

    let bitmap_start = bitmap_phys as usize / PAGE_SIZE;
    let bitmap_pages = bitmap_size.div_ceil(PAGE_SIZE);

    for i in 0..bitmap_pages {
        unsafe {
            bitmap.set(bitmap_start + i, true);
        }

        if free_pages > 0 {
            free_pages -= 1;
        }
    }

    unsafe {
        TOTAL_PAGES = total_pages;
        FREE_PAGES = free_pages;
        BITMAP = Some(bitmap);
    }

    crate::drv::serial::write(b"pmm: initialized\n");
}

pub unsafe fn alloc() -> Option<u64> {
    let bitmap = unsafe { (&raw mut BITMAP).as_mut()? };
    let bitmap = bitmap.as_mut()?;

    let page = unsafe { bitmap.find_free()? };

    unsafe {
        bitmap.set(page, true);

        if FREE_PAGES > 0 {
            FREE_PAGES -= 1;
        }
    }

    Some((page * PAGE_SIZE) as u64)
}

pub unsafe fn free(phys_addr: u64) {
    let page = phys_addr as usize / PAGE_SIZE;

    unsafe {
        if let Some(bitmap) = (&raw mut BITMAP).as_mut().and_then(Option::as_mut) {
            if bitmap.get(page) {
                bitmap.set(page, false);
                FREE_PAGES += 1;
            }
        }
    }
}

pub fn free_pages() -> usize {
    unsafe { FREE_PAGES }
}

pub fn total_pages() -> usize {
    unsafe { TOTAL_PAGES }
}
