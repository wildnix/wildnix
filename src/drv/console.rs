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
        crate::drv::serial::write(bytes);

        let fb_ptr = (&raw mut FRAMEBUFFER).as_mut();
        let font_ptr = (&raw const FONT).as_ref();

        if fb_ptr.is_none() || font_ptr.is_none() {
            return;
        }

        let fb_opt = fb_ptr.unwrap();
        let font_opt = font_ptr.unwrap();

        if fb_opt.is_none() || font_opt.is_none() {
            return;
        }

        let fb = fb_opt.as_mut().unwrap();
        let font = font_opt.as_ref().unwrap();

        fb.write_str(font, bytes, Color::White as u32, Color::Black as u32);
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
