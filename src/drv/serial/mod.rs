pub unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
    );
}

pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
    );
    value
}

pub unsafe fn write(s: &[u8]) {
    for &b in s {
        outb(0x3F8u16, b);
    }
}

pub unsafe fn write_hex(mut value: u64) {
    let mut buf = [0u8; 18];
    buf[0] = b'0';
    buf[1] = b'x';

    for i in 0..16 {
        let shift = 60 - i * 4;
        let nibble = ((value >> shift) & 0xF) as u8;
        buf[2 + i] = match nibble {
            0..=9 => b'0' + nibble,
            _ => b'A' + nibble - 10,
        };
    }

    write(&buf);
}
