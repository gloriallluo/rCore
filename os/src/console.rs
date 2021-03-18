use crate::sbi::sbi_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            sbi_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

macro_rules! impl_color_print {
    ($log_name: expr, $front_color: expr, $back_color: expr,
    $fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(
            concat!(
                "\x1b[", $back_color, ";97m[", $log_name, "]\x1b[0m ",
                "\x1b[1;", $front_color,  "m", $fmt, "\n\x1b[0m"
            )
            $(, $($arg)+)?
        ));
    }
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        impl_color_print!("ERROR", 31, 41, $fmt $(, $($arg)+)?);
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        impl_color_print!("WARN", 33, 43, $fmt $(, $($arg)+)?);
    }
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        impl_color_print!("INFO", 34, 44, $fmt $(, $($arg)+)?);
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        impl_color_print!("DEBUG", 32, 42, $fmt $(, $($arg)+)?);
    }
}

#[macro_export]
macro_rules! trace {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        impl_color_print!("TRACE", 90, 100, $fmt $(, $($arg)+)?);
    }
}
