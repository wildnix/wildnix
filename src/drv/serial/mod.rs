pub unsafe fn write(s: &[u8]) {
    for &b in s {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0x3F8u16,
            in("al") b,
        );
    }
}
