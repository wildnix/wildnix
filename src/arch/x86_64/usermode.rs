pub unsafe fn enter_usermode(entry: u64, stack_top: u64) -> ! {
    unsafe {
        core::arch::asm!(
            "cli",

            "mov ax, {user_data:x}",
            "mov ds, ax",
            "mov es, ax",

            "push {user_data}",
            "push {stack}",
            "pushfq",
            "pop rax",
            "and rax, ~0x200",
            "push rax",
            "push {user_code}",
            "push {entry}",
            "iretq",

            user_data = in(reg) 0x1Bu64,
            user_code = in(reg) 0x23u64,
            stack = in(reg) stack_top,
            entry = in(reg) entry,
            options(noreturn),
        );
    }
}
