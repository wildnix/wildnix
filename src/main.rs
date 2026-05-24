#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod drv;

use limine::request::{ExecutableAddressRequest, FramebufferRequest, HhdmRequest};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

static FONT_DATA: &[u8] = include_bytes!("../assets/ter-u16n.psf");

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
    drv::serial::write(b"kmain reached\nAbout to init arch\n");

    crate::arch::init();
    drv::serial::write(b"arch init done\n");
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

    // Main display code
    drv::serial::write(b"checking framebuffer\n");

    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        drv::serial::write(b"got framebuffer response\n");
        if let Some(fb) = fb_response.framebuffers().first() {
            drv::serial::write(b"got first fb\n");

            let mut display = drv::graphics::Framebuffer::new(
                fb.address() as *mut u8,
                fb.pitch as usize,
                fb.bpp as usize / 8,
                fb.width as usize,
                fb.height as usize,
            );
            drv::serial::write(b"framebuffer created\n");

            let font = match drv::graphics::Font::from_bytes(FONT_DATA.as_ptr()) {
                Some(f) => {
                    drv::serial::write(b"font ok\n");
                    f
                }
                None => {
                    drv::serial::write(b"font invalid\n");
                    panic!("invalid font")
                }
            };

            display.clear(0x00000000);
            drv::serial::write(b"cleared\n");

            display.write_str(&font, b"Wildnix\n", 0xFFFFFFFF, 0x00000000);
            drv::serial::write(b"text written\n");
        } else {
            drv::serial::write(b"no framebuffers in response\n");
        }
    } else {
        drv::serial::write(b"no framebuffer response\n");
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
