#![no_std]
#![no_main]

mod drv;

use limine::request::{ExecutableAddressRequest, FramebufferRequest, HhdmRequest};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

#[used]
#[unsafe(link_section = ".limine_reqs_start")]
static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".limine_reqs_end")]
static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    drv::serial::write(b"kmain reached\n");

    if BASE_REVISION.is_supported() {
        drv::serial::write(b"base revision ok\n");
    } else {
        drv::serial::write(b"base revision NOT supported, continuing anyway\n");
    }

    if let Some(_) = HHDM_REQUEST.response() {
        drv::serial::write(b"hhdm ok\n");
    } else {
        drv::serial::write(b"no hhdm\n");
    }

    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        drv::serial::write(b"got framebuffer\n");
        if let Some(fb) = fb_response.framebuffers().first() {
            let pixels = fb.as_slice_mut();
            for y in 0..100_usize {
                for x in 0..200_usize {
                    let offset = y * fb.pitch as usize + x * (fb.bpp as usize / 8);
                    pixels[offset] = 0xFF;
                    pixels[offset + 1] = 0xFF;
                    pixels[offset + 2] = 0xFF;
                    pixels[offset + 3] = 0xFF;
                }
            }
        }
    } else {
        drv::serial::write(b"no framebuffer\n");
    }

    loop {
        core::arch::asm!("hlt");
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { drv::serial::write(b"PANIC\n") };
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
