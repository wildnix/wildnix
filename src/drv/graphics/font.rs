const PSF2_MAGIC: u32 = 0x864ab572;

#[repr(C, packed)]
struct Psf2Header {
    magic: u32,
    version: u32,
    header_size: u32,
    flags: u32,
    glyph_count: u32,
    glyph_size: u32,
    height: u32,
    width: u32,
}

pub struct Font {
    header: *const Psf2Header,
    glyphs: *const u8,
}

impl Font {
    pub unsafe fn from_bytes(data: *const u8) -> Option<Self> {
        let header = data as *const Psf2Header;
        if (*header).magic != PSF2_MAGIC {
            return None;
        }
        let glyphs = data.add((*header).header_size as usize);
        Some(Self { header, glyphs })
    }

    pub unsafe fn glyph_size(&self) -> usize {
        (*self.header).glyph_size as usize
    }

    pub unsafe fn height(&self) -> usize {
        (*self.header).height as usize
    }

    pub unsafe fn width(&self) -> usize {
        (*self.header).width as usize
    }

    pub unsafe fn glyph(&self, c: u8) -> *const u8 {
        let count = (*self.header).glyph_count as usize;
        let idx = if (c as usize) < count { c as usize } else { 0 };
        self.glyphs.add(idx * self.glyph_size())
    }
}
