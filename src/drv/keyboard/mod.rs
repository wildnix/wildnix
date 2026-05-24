use crate::drv::serial::inb;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const KEYBOARD_QUEUE_SIZE: usize = 256;

static mut SHIFT: bool = false;
static mut KEYBOARD_QUEUE: [u8; KEYBOARD_QUEUE_SIZE] = [0u8; KEYBOARD_QUEUE_SIZE];
static mut KEYBOARD_QUEUE_HEAD: usize = 0;
static mut KEYBOARD_QUEUE_TAIL: usize = 0;

pub fn queue_push(c: u8) {
    unsafe {
        let next_tail = (KEYBOARD_QUEUE_TAIL + 1) % KEYBOARD_QUEUE_SIZE;
        if next_tail != KEYBOARD_QUEUE_HEAD {
            KEYBOARD_QUEUE[KEYBOARD_QUEUE_TAIL] = c;
            KEYBOARD_QUEUE_TAIL = next_tail;
        }
    }
}

pub fn queue_pop() -> Option<u8> {
    unsafe {
        if KEYBOARD_QUEUE_HEAD == KEYBOARD_QUEUE_TAIL {
            None
        } else {
            let c = KEYBOARD_QUEUE[KEYBOARD_QUEUE_HEAD];
            KEYBOARD_QUEUE_HEAD = (KEYBOARD_QUEUE_HEAD + 1) % KEYBOARD_QUEUE_SIZE;
            Some(c)
        }
    }
}

pub fn read_scancode() -> Option<u8> {
    unsafe {
        let status = inb(STATUS_PORT);

        if status & 1 == 0 {
            return None;
        }

        Some(inb(DATA_PORT))
    }
}

pub fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    unsafe {
        match scancode {
            0x2A | 0x36 => {
                SHIFT = true;
                None
            }
            0xAA | 0xB6 => {
                SHIFT = false;
                None
            }

            0x01 => Some(27),
            0x0E => Some(8),
            0x0F => Some(b'\t'),
            0x1C => Some(b'\n'),
            0x39 => Some(b' '),

            _ => {
                if scancode as usize >= 128 {
                    return None;
                }

                let c = if SHIFT {
                    SCANCODE_SET_1_SHIFTED[scancode as usize]
                } else {
                    SCANCODE_SET_1[scancode as usize]
                };

                if c == 0 {
                    None
                } else {
                    Some(c)
                }
            }
        }
    }
}

pub fn read_char() -> Option<u8> {
    let scancode = read_scancode()?;

    if scancode == 0xE0 {
        return None;
    }

    // For now, don't filter releases - let scancode_to_ascii handle it
    scancode_to_ascii(scancode)
}

static SCANCODE_SET_1: [u8; 128] = {
    let mut map = [0u8; 128];

    map[0x02] = b'1';
    map[0x03] = b'2';
    map[0x04] = b'3';
    map[0x05] = b'4';
    map[0x06] = b'5';
    map[0x07] = b'6';
    map[0x08] = b'7';
    map[0x09] = b'8';
    map[0x0A] = b'9';
    map[0x0B] = b'0';
    map[0x0C] = b'-';
    map[0x0D] = b'=';

    map[0x10] = b'q';
    map[0x11] = b'w';
    map[0x12] = b'e';
    map[0x13] = b'r';
    map[0x14] = b't';
    map[0x15] = b'y';
    map[0x16] = b'u';
    map[0x17] = b'i';
    map[0x18] = b'o';
    map[0x19] = b'p';
    map[0x1A] = b'[';
    map[0x1B] = b']';

    map[0x1E] = b'a';
    map[0x1F] = b's';
    map[0x20] = b'd';
    map[0x21] = b'f';
    map[0x22] = b'g';
    map[0x23] = b'h';
    map[0x24] = b'j';
    map[0x25] = b'k';
    map[0x26] = b'l';
    map[0x27] = b';';
    map[0x28] = b'\'';
    map[0x29] = b'`';

    map[0x2B] = b'\\';
    map[0x2C] = b'z';
    map[0x2D] = b'x';
    map[0x2E] = b'c';
    map[0x2F] = b'v';
    map[0x30] = b'b';
    map[0x31] = b'n';
    map[0x32] = b'm';
    map[0x33] = b',';
    map[0x34] = b'.';
    map[0x35] = b'/';

    map
};

static SCANCODE_SET_1_SHIFTED: [u8; 128] = {
    let mut map = [0u8; 128];

    map[0x02] = b'!';
    map[0x03] = b'@';
    map[0x04] = b'#';
    map[0x05] = b'$';
    map[0x06] = b'%';
    map[0x07] = b'^';
    map[0x08] = b'&';
    map[0x09] = b'*';
    map[0x0A] = b'(';
    map[0x0B] = b')';
    map[0x0C] = b'_';
    map[0x0D] = b'+';

    map[0x10] = b'Q';
    map[0x11] = b'W';
    map[0x12] = b'E';
    map[0x13] = b'R';
    map[0x14] = b'T';
    map[0x15] = b'Y';
    map[0x16] = b'U';
    map[0x17] = b'I';
    map[0x18] = b'O';
    map[0x19] = b'P';
    map[0x1A] = b'{';
    map[0x1B] = b'}';

    map[0x1E] = b'A';
    map[0x1F] = b'S';
    map[0x20] = b'D';
    map[0x21] = b'F';
    map[0x22] = b'G';
    map[0x23] = b'H';
    map[0x24] = b'J';
    map[0x25] = b'K';
    map[0x26] = b'L';
    map[0x27] = b':';
    map[0x28] = b'"';
    map[0x29] = b'~';

    map[0x2B] = b'|';
    map[0x2C] = b'Z';
    map[0x2D] = b'X';
    map[0x2E] = b'C';
    map[0x2F] = b'V';
    map[0x30] = b'B';
    map[0x31] = b'N';
    map[0x32] = b'M';
    map[0x33] = b'<';
    map[0x34] = b'>';
    map[0x35] = b'?';

    map
};

pub unsafe fn handle_irq() {
    let scancode = read_scancode();

    crate::drv::serial::write(b"irq1 scancode = ");

    if let Some(sc) = scancode {
        crate::drv::serial::write_hex(sc as u64);
        crate::drv::serial::write(b"\n");
    } else {
        crate::drv::serial::write(b"none\n");
    }
}

pub unsafe fn read_scancode_raw() -> u8 {
    inb(DATA_PORT)
}
