#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod drv;
mod mem;
mod shell;

#[macro_use]
mod macros;

pub mod panic;

extern crate alloc;

use alloc::boxed::Box;
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

use limine::request::{ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemmapRequest};

use crate::drv::graphics::Color;

static FONT_DATA: &[u8] = include_bytes!("../assets/ter-u16n.psf");

static CONSOLE: drv::console::Console = drv::console::Console;

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

    drv::serial::write(b"getting hhdm\n");

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
    unsafe {
        let cs: u16;
        core::arch::asm!("mov {0:x}, cs", out(reg) cs);
        drv::serial::write(b"CS = ");
        drv::serial::write_hex(cs as u64);
        drv::serial::write(b"\n");

        let ss: u16;
        core::arch::asm!("mov {0:x}, ss", out(reg) ss);
        drv::serial::write(b"SS = ");
        drv::serial::write_hex(ss as u64);
        drv::serial::write(b"\n");
    }
    drv::serial::write(b"kernel done\n");
    drv::serial::write(b"arch init done\n");

    drv::serial::write(b"about to init mem\n");
    mem::init(hhdm);
    drv::serial::write(b"mem init done\n");

    let x = Box::new(1337u64);

    drv::serial::write(b"heap test value = ");
    drv::serial::write_hex(*x);
    drv::serial::write(b"\n");

    drv::serial::write(b"checking framebuffer\n");

    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        drv::serial::write(b"got framebuffer response\n");

        if let Some(fb) = fb_response.framebuffers().first() {
            drv::serial::write(b"got first framebuffer\n");

            let font = drv::graphics::Font::from_bytes(FONT_DATA.as_ptr()).expect("font");

            let mut display = drv::graphics::Framebuffer::new(
                fb.address() as *mut u8,
                fb.pitch as usize,
                fb.bpp as usize / 8,
                fb.width as usize,
                fb.height as usize,
            );

            display.clear(Color::Black as u32);

            drv::console::init(display, font);

            drv::serial::write(b"framebuffer created\n");

            let font = match drv::graphics::Font::from_bytes(FONT_DATA.as_ptr()) {
                Some(font) => {
                    drv::serial::write(b"font ok\n");
                    font
                }
                None => {
                    drv::serial::write(b"font invalid\n");
                    panic!("invalid font");
                }
            };
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
            println!("Hello World!");
            println!("Value = {}", 1337);
            drv::serial::write(b"text written\n");
        } else {
            drv::serial::write(b"no framebuffer found\n");
        }
        unsafe {
            let rsp: u64;
            core::arch::asm!("mov {}, rsp", out(reg) rsp);
            drv::serial::write(b"rsp = ");
            drv::serial::write_hex(rsp);
            drv::serial::write(b"\n");
        }
        let mut shell = shell::Shell::new();

        loop {
            shell.tick();
            unsafe { core::arch::asm!("hlt") };
        }
    } else {
        drv::serial::write(b"no framebuffer response\n");
    }
    drv::serial::write(b"kernel done\n");

    loop {
        core::arch::asm!("hlt");
    }
}
