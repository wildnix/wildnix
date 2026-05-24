pub mod heap;
pub mod pmm;
pub mod vmm;

pub unsafe fn init(hhdm_offset: u64) {
    crate::drv::serial::write(b"mem: init\n");

    unsafe {
        pmm::init(hhdm_offset);
        vmm::init(hhdm_offset);
        heap::init();
    }

    crate::drv::serial::write(b"mem: done\n");
}
