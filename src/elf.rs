use crate::mem;

const EI_NIDENT: usize = 16;

const PT_LOAD: u32 = 1;

const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

const PAGE_SIZE: u64 = 4096;

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Header {
    ident: [u8; EI_NIDENT],
    elf_type: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64ProgramHeader {
    p_type: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

pub struct LoadedElf {
    pub entry: u64,
}

fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

unsafe fn read_struct<T: Copy>(data: &[u8], offset: usize) -> Option<T> {
    if offset + core::mem::size_of::<T>() > data.len() {
        return None;
    }

    let ptr = unsafe { data.as_ptr().add(offset) as *const T };
    Some(unsafe { core::ptr::read_unaligned(ptr) })
}

pub unsafe fn load(data: &[u8]) -> Option<LoadedElf> {
    let header: Elf64Header = unsafe { read_struct(data, 0)? };

    if &header.ident[0..4] != b"\x7FELF" {
        return None;
    }

    if header.ident[4] != 2 {
        return None; // ELFCLASS64
    }

    if header.ident[5] != 1 {
        return None; // little endian
    }

    if header.machine != 0x3E {
        return None; // x86_64
    }

    for i in 0..header.phnum {
        let ph_offset = header.phoff as usize + i as usize * header.phentsize as usize;
        let ph: Elf64ProgramHeader = unsafe { read_struct(data, ph_offset)? };

        if ph.p_type != PT_LOAD {
            continue;
        }

        let seg_start = align_down(ph.vaddr, PAGE_SIZE);
        let seg_end = align_up(ph.vaddr + ph.memsz, PAGE_SIZE);

        let mut flags = mem::vmm::FLAG_PRESENT | mem::vmm::FLAG_USER;

        if ph.flags & PF_W != 0 {
            flags |= mem::vmm::FLAG_WRITABLE;
        }

        flags |= mem::vmm::FLAG_WRITABLE;

        for virt in (seg_start..seg_end).step_by(PAGE_SIZE as usize) {
            let phys = unsafe { mem::pmm::alloc()? };

            unsafe {
                mem::vmm::map_page(virt, phys, flags);
            }

            let dst = virt as *mut u8;

            for j in 0..PAGE_SIZE {
                unsafe {
                    dst.add(j as usize).write_volatile(0);
                }
            }
        }

        let src_start = ph.offset as usize;
        let src_end = src_start + ph.filesz as usize;

        if src_end > data.len() {
            return None;
        }

        let dst = ph.vaddr as *mut u8;
        let src = &data[src_start..src_end];

        for i in 0..src.len() {
            unsafe {
                dst.add(i).write_volatile(src[i]);
            }
        }
    }

    Some(LoadedElf {
        entry: header.entry,
    })
}
