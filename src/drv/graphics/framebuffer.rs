use super::Font;

pub struct Framebuffer {
    addr: *mut u8,
    pitch: usize,
    bpp: usize,
    width: usize,
    height: usize,
    cursor_x: usize,
    cursor_y: usize,
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
        let offset = y * self.pitch + x * self.bpp;
        let p = self.addr.add(offset) as *mut u32;
        *p = color;
    }

    unsafe fn scroll(&mut self, font: &Font, bg: u32) {
        let line_h = font.height();
        let scroll_bytes = line_h * self.pitch;
        let total_bytes = self.height * self.pitch;

        core::ptr::copy(
            self.addr.add(scroll_bytes),
            self.addr,
            total_bytes - scroll_bytes,
        );

        let last_row_y = self.height - line_h;
        for y in last_row_y..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, bg);
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
        let bytes_per_row = (width + 7) / 8;

        for row in 0..height {
            for col in 0..width {
                let byte = *glyph.add(row * bytes_per_row + col / 8);
                let bit = 0x80 >> (col % 8);
                let color = if byte & bit != 0 { fg } else { bg };
                self.put_pixel(cx + col, cy + row, color);
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
            _ => {
                if self.cursor_x + glyph_w > self.width {
                    self.cursor_x = 0;
                    self.cursor_y += glyph_h;
                }

                if self.cursor_y + glyph_h > self.height {
                    self.scroll(font, bg);
                }

                self.draw_char(font, c, self.cursor_x, self.cursor_y, fg, bg);
                self.cursor_x += glyph_w;
            }
        }

        if self.cursor_y + glyph_h > self.height {
            self.scroll(font, bg);
        }
    }

    pub unsafe fn write_str(&mut self, font: &Font, s: &[u8], fg: u32, bg: u32) {
        for &c in s {
            self.write_char(font, c, fg, bg);
        }
    }

    pub unsafe fn draw_str(&mut self, font: &Font, s: &[u8], x: usize, y: usize, fg: u32, bg: u32) {
        let glyph_w = font.width();
        for (i, &c) in s.iter().enumerate() {
            self.draw_char(font, c, x + i * glyph_w, y, fg, bg);
        }
    }
}
