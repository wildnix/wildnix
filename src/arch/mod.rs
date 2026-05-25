pub mod interrupts;
pub mod x86_64;

pub fn init() {
    x86_64::init();

    unsafe {
        interrupts::pic_remap();

        // Mask all IRQs, then unmask keyboard IRQ1 so IRQ handler runs when enabled locally.
        crate::drv::serial::outb(0x21, 0xFF);
        crate::drv::serial::outb(0xA1, 0xFF);
        interrupts::pic_clear_mask(1);
    }

    unsafe { crate::drv::serial::write(b"arch: initialized\n") };
}
