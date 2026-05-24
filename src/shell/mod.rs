use crate::{clear, drv::keyboard};

const MAX_INPUT: usize = 256;

pub struct Shell {
    buf: [u8; MAX_INPUT],
    len: usize,
}

impl Shell {
    pub fn new() -> Self {
        crate::drv::console::write_bytes(b"wildnix$ ");
        Self {
            buf: [0u8; MAX_INPUT],
            len: 0,
        }
    }

    pub fn tick(&mut self) {
        unsafe { crate::drv::serial::write(b"tick\n") };

        let Some(c) = (unsafe { keyboard::read_char() }) else {
            return;
        };

        match c {
            b'\n' => {
                crate::drv::console::write_bytes(b"\n");
                self.execute();
                self.buf = [0u8; MAX_INPUT];
                self.len = 0;
                crate::drv::console::write_bytes(b"wildnix$ ");
            }
            0x08 => {
                // backspace
                if self.len > 0 {
                    self.len -= 1;
                    crate::drv::console::write_bytes(b"\x08");
                }
            }
            c => {
                if self.len < MAX_INPUT - 1 {
                    self.buf[self.len] = c;
                    self.len += 1;
                    crate::drv::console::write_bytes(&[c]);
                }
            }
        }
    }

    fn execute(&mut self) {
        let cmd = &self.buf[..self.len];

        match cmd {
            b"help" => {
                crate::drv::console::write_bytes(b"commands: help, clear, version\n");
            }
            b"clear" => {
                clear!();
                crate::drv::console::reset_cursor();
            }
            b"version" => {
                crate::drv::console::write_bytes(b"WildNIX v0.1.0\n");
            }
            b"" => {}
            _ => {
                crate::drv::console::write_bytes(b"unknown command: ");
                crate::drv::console::write_bytes(cmd);
                crate::drv::console::write_bytes(b"\n");
            }
        }
    }
}
