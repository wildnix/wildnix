use core::fmt::{self, Write};

use crate::drv::graphics::{Color, Font, Framebuffer};

static mut FRAMEBUFFER: Option<Framebuffer> = None;
static mut FONT: Option<Font> = None;

pub unsafe fn init(framebuffer: Framebuffer, font: Font) {
    unsafe {
        FRAMEBUFFER = Some(framebuffer);
        FONT = Some(font);
    }
}

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_bytes(s.as_bytes());
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut console = Console;
    let _ = console.write_fmt(args);
}

pub fn _clear() {
    unsafe {
        if let Some(fb) = (&raw mut FRAMEBUFFER).as_mut().and_then(Option::as_mut) {
            fb.clear(Color::Black as u32);
        }
    }
}

pub fn write_bytes(bytes: &[u8]) {
    unsafe {
        if let (Some(fb), Some(font)) = (
            (&raw mut FRAMEBUFFER).as_mut().and_then(Option::as_mut),
            (&raw const FONT).as_ref().and_then(Option::as_ref),
        ) {
            crate::drv::serial::write(b"fb addr=");
            crate::drv::serial::write_hex(fb.addr as u64);
            crate::drv::serial::write(b" pitch=");
            crate::drv::serial::write_hex(fb.pitch as u64);
            crate::drv::serial::write(b" w=");
            crate::drv::serial::write_hex(fb.width as u64);
            crate::drv::serial::write(b" h=");
            crate::drv::serial::write_hex(fb.height as u64);
            crate::drv::serial::write(b"\n");
            fb.write_str(font, bytes, Color::White as u32, Color::Black as u32);
        }
    }
}

pub fn reset_cursor() {
    unsafe {
        if let Some(fb) = (&raw mut FRAMEBUFFER).as_mut().and_then(Option::as_mut) {
            fb.cursor_x = 0;
            fb.cursor_y = 0;
        }
    }
}
