use crate::mem;

const PAGE_SIZE: u64 = 4096;

pub const USER_CODE_ADDR: u64 = 0x0000_0000_0040_0000;
pub const USER_STACK_TOP: u64 = 0x0000_0000_0080_0000;

// user program:
// loop:
//   xor rax, rax
//   syscall
//   jmp loop
static USER_CODE: &[u8] = &[
    0x48, 0x31, 0xC0, // xor rax, rax
    0x0F, 0x05, // syscall
    0xEB, 0xF9, // jmp -7
];

pub unsafe fn setup_user() {
    let code_phys = unsafe { mem::pmm::alloc().expect("failed to allocate user code page") };

    let stack_phys = unsafe { mem::pmm::alloc().expect("failed to allocate user stack page") };

    unsafe {
        mem::vmm::map_page(
            USER_CODE_ADDR,
            code_phys,
            mem::vmm::FLAG_PRESENT | mem::vmm::FLAG_WRITABLE | mem::vmm::FLAG_USER,
        );

        mem::vmm::map_page(
            USER_STACK_TOP - PAGE_SIZE,
            stack_phys,
            mem::vmm::FLAG_PRESENT | mem::vmm::FLAG_WRITABLE | mem::vmm::FLAG_USER,
        );
    }

    let dst = USER_CODE_ADDR as *mut u8;

    for i in 0..USER_CODE.len() {
        unsafe {
            dst.add(i).write_volatile(USER_CODE[i]);
        }
    }

    crate::drv::serial::write(b"user: setup done\n");
}
