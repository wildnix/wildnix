#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Color {
    Black = 0x000000,
    Blue = 0x0000AA,
    Green = 0x00AA00,
    Cyan = 0x00AAAA,
    Red = 0xAA0000,
    Magenta = 0xAA00AA,
    Brown = 0xAA5500,
    LightGray = 0xAAAAAA,
    DarkGray = 0x555555,
    LightBlue = 0x5555FF,
    LightGreen = 0x55FF55,
    LightCyan = 0x55FFFF,
    LightRed = 0xFF5555,
    LightMagenta = 0xFF55FF,
    Yellow = 0xFFFF55,
    White = 0xFFFFFF,
}

impl Color {
    pub fn ansi_fg(code: u8) -> Option<u32> {
        Some(match code {
            30 => Self::Black as u32,
            31 => Self::Red as u32,
            32 => Self::Green as u32,
            33 => Self::Brown as u32,
            34 => Self::Blue as u32,
            35 => Self::Magenta as u32,
            36 => Self::Cyan as u32,
            37 => Self::LightGray as u32,

            90 => Self::DarkGray as u32,
            91 => Self::LightRed as u32,
            92 => Self::LightGreen as u32,
            93 => Self::Yellow as u32,
            94 => Self::LightBlue as u32,
            95 => Self::LightMagenta as u32,
            96 => Self::LightCyan as u32,
            97 => Self::White as u32,

            _ => return None,
        })
    }

    pub fn ansi_bg(code: u8) -> Option<u32> {
        Some(match code {
            40 => Self::Black as u32,
            41 => Self::Red as u32,
            42 => Self::Green as u32,
            43 => Self::Brown as u32,
            44 => Self::Blue as u32,
            45 => Self::Magenta as u32,
            46 => Self::Cyan as u32,
            47 => Self::LightGray as u32,

            100 => Self::DarkGray as u32,
            101 => Self::LightRed as u32,
            102 => Self::LightGreen as u32,
            103 => Self::Yellow as u32,
            104 => Self::LightBlue as u32,
            105 => Self::LightMagenta as u32,
            106 => Self::LightCyan as u32,
            107 => Self::White as u32,

            _ => return None,
        })
    }
}
