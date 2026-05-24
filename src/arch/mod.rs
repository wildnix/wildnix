pub mod interrupts;
pub mod x86_64;

pub fn init() {
    x86_64::init();

    unsafe {
        interrupts::pic_remap();

        crate::drv::serial::outb(0x21, 0xFF);
        crate::drv::serial::outb(0xA1, 0xFF);

        // interrupts::pic_clear_mask(1);

        // core::arch::asm!("sti");
    }

    unsafe { crate::drv::serial::write(b"arch: initialized\n") };
}
