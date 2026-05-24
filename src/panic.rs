use crate::drv;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        drv::serial::write(b"PANIC\n");
    }

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
