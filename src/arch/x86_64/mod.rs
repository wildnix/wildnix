pub mod gdt;
pub mod idt;
pub mod syscall;
pub mod usermode;

pub fn init() {
    gdt::init();
    idt::init();

    unsafe {
        syscall::init();
    }
}
