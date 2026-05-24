use crate::mem;

const EI_NIDENT: usize = 16;
const PT_LOAD: u32 = 1;

const PF_W: u32 = 2;

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

fn align_down(v: u64, a: u64) -> u64 {
    v & !(a - 1)
}

fn align_up(v: u64, a: u64) -> u64 {
    (v + a - 1) & !(a - 1)
}

unsafe fn read_struct<T: Copy>(data: &[u8], offset: usize) -> Option<T> {
    if offset + core::mem::size_of::<T>() > data.len() {
        return None;
    }

    Some(core::ptr::read_unaligned(
        data.as_ptr().add(offset) as *const T
    ))
}

pub unsafe fn load(data: &[u8]) -> Option<LoadedElf> {
    let header: Elf64Header = read_struct(data, 0)?;

    if &header.ident[0..4] != b"\x7FELF" {
        return None;
    }

    for i in 0..header.phnum {
        let ph_off = header.phoff as usize + i as usize * header.phentsize as usize;

        let ph: Elf64ProgramHeader = read_struct(data, ph_off)?;

        if ph.p_type != PT_LOAD {
            continue;
        }

        let seg_start = align_down(ph.vaddr, PAGE_SIZE);
        let seg_end = align_up(ph.vaddr + ph.memsz, PAGE_SIZE);

        let mut flags = mem::vmm::FLAG_PRESENT | mem::vmm::FLAG_USER;

        if ph.flags & PF_W != 0 {
            flags |= mem::vmm::FLAG_WRITABLE;
        }

        for virt in (seg_start..seg_end).step_by(PAGE_SIZE as usize) {
            let phys = mem::pmm::alloc()?;

            mem::vmm::map_page(virt, phys, flags | mem::vmm::FLAG_WRITABLE);

            let page = virt as *mut u8;

            for j in 0..PAGE_SIZE {
                page.add(j as usize).write_volatile(0);
            }
        }

        let file = &data[ph.offset as usize..(ph.offset + ph.filesz) as usize];

        let src_start = ph.offset as usize;
        let src_end = src_start + ph.filesz as usize;

        if src_end > data.len() {
            return None;
        }

        let src = &data[src_start..src_end];
        let dst = ph.vaddr as *mut u8;

        for i in 0..src.len() {
            dst.add(i).write_volatile(src[i]);
        }
        let b = *(header.entry as *const u8);

        crate::drv::serial::write(b"entry byte = ");
        crate::drv::serial::write_hex(b as u64);
        crate::drv::serial::write(b"\n");
        crate::drv::serial::write(b"loaded segment\n");
    }

    crate::drv::serial::write(b"elf load done\n");

    Some(LoadedElf {
        entry: header.entry,
    })
}
