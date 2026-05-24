use super::Font;

pub struct Framebuffer {
    pub addr: *mut u8,
    pub pitch: usize,
    pub bpp: usize,
    pub width: usize,
    pub height: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl Framebuffer {
    pub unsafe fn new(
        addr: *mut u8,
        pitch: usize,
        bpp: usize,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            addr,
            pitch,
            bpp,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub unsafe fn clear(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, color);
            }
        }

        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub unsafe fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = y * self.pitch + x * self.bpp;
        let p = unsafe { self.addr.add(offset) };

        unsafe {
            match self.bpp {
                4 => {
                    p.add(0).write_volatile((color & 0xFF) as u8);
                    p.add(1).write_volatile(((color >> 8) & 0xFF) as u8);
                    p.add(2).write_volatile(((color >> 16) & 0xFF) as u8);
                    p.add(3).write_volatile(((color >> 24) & 0xFF) as u8);
                }
                3 => {
                    p.add(0).write_volatile((color & 0xFF) as u8);
                    p.add(1).write_volatile(((color >> 8) & 0xFF) as u8);
                    p.add(2).write_volatile(((color >> 16) & 0xFF) as u8);
                }
                _ => {}
            }
        }
    }

    unsafe fn scroll(&mut self, font: &Font, bg: u32) {
        let line_h = font.height();
        let scroll_bytes = line_h * self.pitch;
        let total_bytes = self.height * self.pitch;

        unsafe {
            core::ptr::copy(
                self.addr.add(scroll_bytes),
                self.addr,
                total_bytes - scroll_bytes,
            );
        }

        let last_row_y = self.height - line_h;

        for y in last_row_y..self.height {
            for x in 0..self.width {
                unsafe {
                    self.put_pixel(x, y, bg);
                }
            }
        }

        if self.cursor_y >= line_h {
            self.cursor_y -= line_h;
        } else {
            self.cursor_y = 0;
        }
    }

    pub unsafe fn draw_char(&mut self, font: &Font, c: u8, cx: usize, cy: usize, fg: u32, bg: u32) {
        let glyph = font.glyph(c);
        let height = font.height();
        let width = font.width();
        let bytes_per_row = width.div_ceil(8);

        for row in 0..height {
            for col in 0..width {
                let byte = unsafe { *glyph.add(row * bytes_per_row + col / 8) };
                let bit = 0x80 >> (col % 8);
                let color = if byte & bit != 0 { fg } else { bg };

                unsafe {
                    self.put_pixel(cx + col, cy + row, color);
                }
            }
        }
    }

    pub unsafe fn write_char(&mut self, font: &Font, c: u8, fg: u32, bg: u32) {
        let glyph_w = font.width();
        let glyph_h = font.height();

        match c {
            b'\n' => {
                self.cursor_x = 0;
                self.cursor_y += glyph_h;
            }
            b'\r' => {
                self.cursor_x = 0;
            }
            b'\t' => {
                let spaces = 4 - (self.cursor_x / glyph_w) % 4;
                self.cursor_x += spaces * glyph_w;
            }
            b'\x08' => {
                if self.cursor_x >= glyph_w {
                    self.cursor_x -= glyph_w;
                } else if self.cursor_y >= glyph_h {
                    self.cursor_x = (self.width / glyph_w - 1) * glyph_w;
                    self.cursor_y -= glyph_h;
                }

                self.clear_char(self.cursor_x, self.cursor_y, glyph_w, glyph_h, bg);
            }
            _ => {
                if self.cursor_x + glyph_w > self.width {
                    self.cursor_x = 0;
                    self.cursor_y += glyph_h;
                }

                if self.cursor_y + glyph_h > self.height {
                    unsafe {
                        self.scroll(font, bg);
                    }
                }

                unsafe {
                    self.draw_char(font, c, self.cursor_x, self.cursor_y, fg, bg);
                }

                self.cursor_x += glyph_w;
            }
        }

        if self.cursor_y + glyph_h > self.height {
            unsafe {
                self.scroll(font, bg);
            }
        }
    }

    pub unsafe fn clear_char(&mut self, x: usize, y: usize, width: usize, height: usize, bg: u32) {
        for py in 0..height {
            for px in 0..width {
                self.put_pixel(x + px, y + py, bg);
            }
        }
    }

    pub unsafe fn write_str(&mut self, font: &Font, s: &[u8], mut fg: u32, mut bg: u32) {
        let default_fg = fg;
        let default_bg = bg;

        let mut i = 0;

        while i < s.len() {
            if s[i] == 0x1B {
                if i + 1 < s.len() && s[i + 1] == b'[' {
                    i += 2;

                    let mut code: u16 = 0;

                    while i < s.len() {
                        let c = s[i];

                        if c.is_ascii_digit() {
                            code = code * 10 + (c - b'0') as u16;
                        } else if c == b'm' {
                            match code {
                                0 => {
                                    fg = default_fg;
                                    bg = default_bg;
                                }
                                n => {
                                    if let Some(color) = super::Color::ansi_fg(n as u8) {
                                        fg = color;
                                    }

                                    if let Some(color) = super::Color::ansi_bg(n as u8) {
                                        bg = color;
                                    }
                                }
                            }

                            break;
                        }

                        i += 1;
                    }
                }
            } else {
                self.write_char(font, s[i], fg, bg);
            }

            i += 1;
        }
    }

    pub unsafe fn draw_str(&mut self, font: &Font, s: &[u8], x: usize, y: usize, fg: u32, bg: u32) {
        let glyph_w = font.width();

        for (i, &c) in s.iter().enumerate() {
            unsafe {
                self.draw_char(font, c, x + i * glyph_w, y, fg, bg);
            }
        }
    }
}
