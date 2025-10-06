#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if $crate::logger::is_enabled() {
            println!($($arg)*);
        }
    };
}
