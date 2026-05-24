#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::drv::console::_print(
            format_args!($($arg)*)
        )
    }};
}

#[macro_export]
macro_rules! clear {
    () => {{
        $crate::drv::console::_clear()
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n")
    }};

    ($fmt:expr) => {{
        $crate::print!(concat!($fmt, "\n"))
    }};

    ($fmt:expr, $($arg:tt)*) => {{
        $crate::print!(
            concat!($fmt, "\n"),
            $($arg)*
        )
    }};
}
