#[macro_export]
macro_rules! verbose_println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        let v = crate::VERBOSE.read().unwrap();
        if *v {
            println!($($arg)*);
        }
    }};
}
