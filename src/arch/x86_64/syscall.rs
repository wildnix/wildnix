use core::arch::{asm, naked_asm};

use crate::{VFS, println};

const IA32_EFER: u32 = 0xC000_0080;
const IA32_STAR: u32 = 0xC000_0081;
const IA32_LSTAR: u32 = 0xC000_0082;
const IA32_FMASK: u32 = 0xC000_0084;

const EFER_SCE: u64 = 1;

pub const SYS_DEBUG: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_READ_KEY: u64 = 2;
pub const SYS_EXIT: u64 = 3;

pub const SYS_FS_READ: u64 = 10;
pub const SYS_FS_WRITE: u64 = 11;
pub const SYS_FS_CREATE: u64 = 12;
pub const SYS_FS_DELETE: u64 = 13;
pub const SYS_FS_LIST: u64 = 14;
pub const SYS_FS_EXISTS: u64 = 15;

static mut SYSCALL_STACK: [u8; 4096 * 4] = [0; 4096 * 4];
static mut USER_RSP_TMP: u64 = 0;

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
        wrmsr(IA32_LSTAR, syscall_entry as *const () as u64);

        // STAR encoding for SYSCALL/SYSRET:
        // - bits 47:32 = kernel CS selector
        // - bits 63:48 = user selector base used by SYSRET
        //   (user CS = base + 0x10, user SS = base + 0x08)
        // With GDT user SS=0x1B and user CS=0x23, base is 0x13.
        let star = ((0x08u64) << 32) | ((0x13u64) << 48);
        wrmsr(IA32_STAR, star);

        // clear IF on syscall entry
        wrmsr(IA32_FMASK, 1 << 9);
    }

    crate::drv::serial::write(b"syscall: initialized\n");
}

#[unsafe(naked)]
extern "C" fn syscall_entry() -> ! {
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

            "mov [rip + {user_rsp_tmp}], rsp",

            "lea rsp, [rip + {stack}]",
            "add rsp, {stack_size}",
            "and rsp, -16",

            "push rcx",
            "push r11",
            "sub rsp, 8",

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

            "add rsp, 8",
            "pop r11",
            "pop rcx",
            "mov rsp, [rip + {user_rsp_tmp}]",

            "sysretq",

            stack = sym SYSCALL_STACK,
            stack_size = const 4096 * 4,
            user_rsp_tmp = sym USER_RSP_TMP,
            handler = sym syscall_handler,
    );
}

#[no_mangle]
extern "C" fn syscall_handler(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    _arg5: u64,
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

        SYS_FS_READ => sys_fs_read(arg1, arg2, arg3, arg4),
        SYS_FS_WRITE => sys_fs_write(arg1, arg2, arg3, arg4),
        SYS_FS_CREATE => sys_fs_create(arg1, arg2),
        SYS_FS_DELETE => sys_fs_delete(arg1, arg2),
        SYS_FS_LIST => sys_fs_list(arg1, arg2, arg3, arg4),
        SYS_FS_EXISTS => sys_fs_exists(arg1, arg2),

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
    loop {
        if let Some(c) = crate::drv::keyboard::queue_pop() {
            return c as u64;
        }

        // Enable interrupts briefly and halt until the next interrupt then disable
        unsafe { core::arch::asm!("sti; hlt; cli"); }
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
        unsafe { core::arch::asm!("hlt"); }
    }
}

fn sys_fs_read(
    path_ptr: u64,
    path_len: u64,
    buf_ptr: u64,
    buf_len: u64,
) -> u64 {
    if path_ptr == 0 || path_len == 0 || buf_ptr == 0 || buf_len == 0 {
        return u64::MAX;
    }

    let path_bytes = unsafe {
        core::slice::from_raw_parts(path_ptr as *const u8, path_len as usize)
    };

    let path = match core::str::from_utf8(path_bytes) {
        Ok(path) => path,
        Err(_) => return u64::MAX,
    };

    let file = unsafe {
        let vfs = (&raw const crate::VFS).as_ref().unwrap().assume_init_ref();

        match vfs.read_file(path) {
            Some(data) => data,
            None => return u64::MAX,
        }
    };

    let copy_len = core::cmp::min(file.len(), buf_len as usize);

    let dst = buf_ptr as *mut u8;

    for i in 0..copy_len {
        unsafe {
            dst.add(i).write_volatile(file[i]);
        }
    }

    copy_len as u64
}

fn sys_fs_list(
    path_ptr: u64,
    path_len: u64,
    buf_ptr: u64,
    buf_len: u64,
) -> u64 {
    if path_ptr == 0 || path_len == 0 || buf_ptr == 0 || buf_len == 0 {
        return u64::MAX;
    }

    let path_bytes = unsafe {
        core::slice::from_raw_parts(path_ptr as *const u8, path_len as usize)
    };

    let path = match core::str::from_utf8(path_bytes) {
        Ok(path) => path,
        Err(_) => return u64::MAX,
    };

    let files = unsafe {
        let vfs = (&raw const crate::VFS)
            .as_ref()
            .unwrap()
            .assume_init_ref();

        vfs.list_dir(path)
    };

    let dst = buf_ptr as *mut u8;
    let mut written = 0usize;

    for name in files {
        let bytes = name.as_bytes();

        for &b in bytes {
            if written >= buf_len as usize {
                return written as u64;
            }

            unsafe {
                dst.add(written).write_volatile(b);
            }

            written += 1;
        }

        if written >= buf_len as usize {
            return written as u64;
        }

        unsafe {
            dst.add(written).write_volatile(b'\n');
        }

        written += 1;
    }

    written as u64
}

fn user_str<'a>(ptr: u64, len: u64) -> Result<&'a str, u64> {
    if ptr == 0 || len == 0 {
        return Err(u64::MAX);
    }

    let bytes = unsafe {
        core::slice::from_raw_parts(ptr as *const u8, len as usize)
    };

    core::str::from_utf8(bytes).map_err(|_| u64::MAX)
}

fn user_bytes<'a>(ptr: u64, len: u64) -> Result<&'a [u8], u64> {
    if ptr == 0 || len == 0 {
        return Err(u64::MAX);
    }

    Ok(unsafe {
        core::slice::from_raw_parts(ptr as *const u8, len as usize)
    })
}

fn sys_fs_write(
    path_ptr: u64,
    path_len: u64,
    data_ptr: u64,
    data_len: u64,
) -> u64 {
    let path = match user_str(path_ptr, path_len) {
        Ok(path) => path,
        Err(e) => return e,
    };

    let data = match user_bytes(data_ptr, data_len) {
        Ok(data) => data,
        Err(e) => return e,
    };

    unsafe {
        let vfs = (&raw mut VFS)
            .as_mut()
            .unwrap()
            .assume_init_mut();

        if !vfs.exists(path) {
            vfs.create_file(path);
        }

        if vfs.write_file(path, data) {
            data_len
        } else {
            u64::MAX
        }
    }
}

fn sys_fs_create(path_ptr: u64, path_len: u64) -> u64 {
    let path = match user_str(path_ptr, path_len) {
        Ok(path) => path,
        Err(e) => return e,
    };

    unsafe {
        let vfs = (&raw mut VFS)
            .as_mut()
            .unwrap()
            .assume_init_mut();

        if vfs.create_file(path) {
            0
        } else {
            u64::MAX
        }
    }
}

fn sys_fs_delete(path_ptr: u64, path_len: u64) -> u64 {
    let path = match user_str(path_ptr, path_len) {
        Ok(path) => path,
        Err(e) => return e,
    };

    unsafe {
        let vfs = (&raw mut VFS)
            .as_mut()
            .unwrap()
            .assume_init_mut();

        if vfs.remove(path) {
            0
        } else {
            u64::MAX
        }
    }
}

fn sys_fs_exists(path_ptr: u64, path_len: u64) -> u64 {
    let path = match user_str(path_ptr, path_len) {
        Ok(path) => path,
        Err(e) => return e,
    };

    unsafe {
        let vfs = (&raw const VFS)
            .as_ref()
            .unwrap()
            .assume_init_ref();

        if vfs.exists(path) {
            1
        } else {
            0
        }
    }
}