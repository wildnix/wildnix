#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod drv;
mod elf;
mod mem;
mod user;

#[macro_use]
mod macros;

pub mod panic;

extern crate alloc;

use limine::request::{ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemmapRequest};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

use crate::drv::graphics::Color;
use core::mem::MaybeUninit;

static FONT_DATA: &[u8] = include_bytes!("../assets/ter-u16n.psf");
static USER_ELF: &[u8] = include_bytes!("../build/userland/init.elf");

static mut VFS: MaybeUninit<drv::fs::vfs::Vfs> = MaybeUninit::uninit();

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
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

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
        drv::serial::write(b"base revision NOT supported\n");
    }

    let hhdm = match HHDM_REQUEST.response() {
        Some(response) => {
            drv::serial::write(b"hhdm ok\n");
            drv::serial::write(b"hhdm offset = ");
            drv::serial::write_hex(response.offset);
            drv::serial::write(b"\n");
            response.offset
        }
        None => {
            drv::serial::write(b"hhdm none\n");
            panic!("no hhdm response");
        }
    };

    drv::serial::write(b"about to init arch\n");
    arch::init();
    drv::serial::write(b"arch init done\n");

    drv::serial::write(b"about to init mem\n");
    mem::init(hhdm);
    drv::serial::write(b"mem init done\n");

    init_framebuffer();

    unsafe {
        user::setup_user_stack();

        drv::serial::write(b"loading init ELF\n");

        let loaded = elf::load(USER_ELF).expect("failed to load user ELF");

        drv::serial::write(b"loaded ELF entry = ");
        drv::serial::write_hex(loaded.entry);
        drv::serial::write(b"\n");

        drv::serial::write(b"entering init-rs\n");

        arch::x86_64::usermode::enter_usermode(loaded.entry, user::USER_STACK_TOP);
    }
}

fn init_framebuffer() {
    unsafe { drv::serial::write(b"checking framebuffer\n") };

    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        unsafe { drv::serial::write(b"got framebuffer response\n") };

        if let Some(fb) = fb_response.framebuffers().first() {
            unsafe { drv::serial::write(b"got first framebuffer\n") };

            let font = unsafe {
                drv::graphics::Font::from_bytes(FONT_DATA.as_ptr()).expect("invalid font")
            };

            let mut display = unsafe {
                drv::graphics::Framebuffer::new(
                    fb.address() as *mut u8,
                    fb.pitch as usize,
                    fb.bpp as usize / 8,
                    fb.width as usize,
                    fb.height as usize,
                )
            };

            unsafe {
                display.clear(Color::Black as u32);
                drv::console::init(display, font);
            }

            unsafe { drv::serial::write(b"framebuffer created\n") };

            println!(
                r#"
          ,--.,--.
 ,-.     /  //  /,-.    ,--.   ,--.,--.,--.   ,--.,--.  ,--.,--.,--.   ,--.
 \  \   /  //  //  /    |  |   |  |`--'|  | ,-|  ||  ,'.|  ||  | \  `.'  /
  \  \ /  //  //  /     |  |.'.|  |,--.|  |' .-. ||  |' '  ||  |  .'    \
  /  //  //  / \  \     |   ,'.   ||  ||  |\ `-' ||  | `   ||  | /  .'.  \
 /  //  //  /   \  \    '--'   '--'`--'`--' `---' `--'  `--'`--''--'   '--'
 `-'`--'`--'     `-'
"#
            );

            println!("WildNIX Operating System v{}", env!("CARGO_PKG_VERSION"));
        } else {
            unsafe { drv::serial::write(b"no framebuffer found\n") };
        }
    } else {
        unsafe { drv::serial::write(b"no framebuffer response\n") };
    }
}
