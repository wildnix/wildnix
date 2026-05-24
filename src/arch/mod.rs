pub mod interrupts;
pub mod x86_64;

pub fn init() {
    x86_64::init();
    unsafe {
        interrupts::pic_clear_mask(1); // unmask keyboard only
    }
    crate::println!("arch: initialized");
}
