use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag controlling whether logs are enabled.
pub static ENABLE_LOGS: AtomicBool = AtomicBool::new(false);

/// Initialize logging from the environment variable.
/// 
/// Set `HTTP_SERVER_LOGS=1` or `HTTP_SERVER_LOGS=true` to enable logging.
pub fn init_logging() {
    let enable = env::var("HTTP_SERVER_LOGS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    println!("log enabled: {:?}", enable);

    ENABLE_LOGS.store(enable, Ordering::Relaxed);
}

/// Internal helper used by the log! macro.
pub fn is_enabled() -> bool {
    ENABLE_LOGS.load(Ordering::Relaxed)
}
