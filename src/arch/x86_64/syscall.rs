use core::arch::{asm, naked_asm};

use crate::println;

const IA32_EFER: u32 = 0xC000_0080;
const IA32_STAR: u32 = 0xC000_0081;
const IA32_LSTAR: u32 = 0xC000_0082;
const IA32_FMASK: u32 = 0xC000_0084;

const EFER_SCE: u64 = 1;

pub const SYS_DEBUG: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_READ_KEY: u64 = 2;
pub const SYS_EXIT: u64 = 3;

static mut SYSCALL_STACK: [u8; 4096 * 4] = [0; 4096 * 4];

unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;

    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }

    ((high as u64) << 32) | low as u64
}

unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;

    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
}

pub unsafe fn init() {
    let efer = unsafe { rdmsr(IA32_EFER) };

    unsafe {
        wrmsr(IA32_EFER, efer | EFER_SCE);
        wrmsr(IA32_LSTAR, syscall_entry as u64);

        // GDT:
        // 0x08 kernel code
        // 0x10 kernel data
        // 0x18 user data
        // 0x20 user code
        let star = ((0x08u64) << 32) | ((0x18u64) << 48);
        wrmsr(IA32_STAR, star);

        // clear IF on syscall entry
        wrmsr(IA32_FMASK, 1 << 9);
    }

    crate::drv::serial::write(b"syscall: initialized\n");
}

#[unsafe(naked)]
extern "C" fn syscall_entry() -> ! {
    unsafe {
        naked_asm!(
            // syscall state:
            // rax = syscall num
            // rdi = arg1
            // rsi = arg2
            // rdx = arg3
            // r10 = arg4
            // r8  = arg5
            // r9  = arg6
            // rcx = user RIP
            // r11 = user RFLAGS

            "mov r12, rsp",

            "lea rsp, [rip + {stack}]",
            "add rsp, {stack_size}",
            "and rsp, -16",

            "push r12",
            "push rcx",
            "push r11",

            // syscall_handler(num, arg1, arg2, arg3, arg4, arg5, arg6)
            // SysV ABI args:
            // rdi, rsi, rdx, rcx, r8, r9
            //
            // Need to move:
            // num  -> rdi
            // arg1 -> rsi
            // arg2 -> rdx
            // arg3 -> rcx
            // arg4 -> r8
            // arg5 -> r9
            // arg6 ignored for now

            "mov rcx, rdx", // arg3 -> rcx
            "mov rdx, rsi", // arg2 -> rdx
            "mov rsi, rdi", // arg1 -> rsi
            "mov rdi, rax", // num  -> rdi
            "mov r8, r10",  // arg4 -> r8
            // r9 already arg5

            "call {handler}",

            "pop r11",
            "pop rcx",
            "pop r12",

            "mov rsp, r12",

            "sysretq",

            stack = sym SYSCALL_STACK,
            stack_size = const 4096 * 4,
            handler = sym syscall_handler,
        );
    }
}

#[no_mangle]
extern "C" fn syscall_handler(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> u64 {
    match num {
        SYS_DEBUG => {
            unsafe {
                crate::drv::serial::write(b"syscall from ring3\n");
            }
            0
        }

        SYS_WRITE => sys_write(arg1, arg2),

        SYS_READ_KEY => sys_read_key(),

        SYS_EXIT => sys_exit(arg1),

        _ => u64::MAX,
    }
}

fn sys_write(ptr: u64, len: u64) -> u64 {
    if ptr == 0 || len == 0 {
        return 0;
    }

    let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };

    crate::drv::console::write_bytes(bytes);

    len
}

fn sys_read_key() -> u64 {
    match crate::drv::keyboard::read_char() {
        Some(c) => c as u64,
        None => 0,
    }
}

fn sys_exit(code: u64) -> u64 {
    unsafe {
        crate::drv::serial::write(b"process exited with code ");
        crate::drv::serial::write_hex(code);
        crate::drv::serial::write(b"\n");
        println!("process exited with code {}", code);
    }

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
