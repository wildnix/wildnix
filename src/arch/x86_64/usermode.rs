use core::arch::asm;

pub const USER_DATA_SELECTOR: u64 = 0x1B;
pub const USER_CODE_SELECTOR: u64 = 0x23;

pub unsafe fn enter_usermode(entry: u64, stack_top: u64) -> ! {
    unsafe {
        asm!(
            "cli",

            "mov ax, {user_data:x}",
            "mov ds, ax",
            "mov es, ax",

            "push {user_data}",
            "push {stack}",
            "pushfq",
            "pop rax",
            "or rax, 0x200",
            "push rax",
            "push {user_code}",
            "push {entry}",
            "iretq",

            user_data = in(reg) USER_DATA_SELECTOR,
            user_code = in(reg) USER_CODE_SELECTOR,
            stack = in(reg) stack_top,
            entry = in(reg) entry,

            options(noreturn)
        );
    }
}
