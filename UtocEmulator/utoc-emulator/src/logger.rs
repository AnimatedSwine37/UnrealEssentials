use std::sync::OnceLock;

#[repr(u32)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum LogLevel {
    /// Contains information generally only useful to people working on emulators.
    Debug,
    /// Anything that may be noteworthy to the user.
    Information,
    /// Something interesting happened, pay attention.
    Warning,
    /// Something went wrong, hopefully we can recover.
    Error,
    /// Application will probably crash.
    Fatal,
}

#[macro_export]
macro_rules! log {
    ($ty:ident, $($fmt:tt)*) => {
        let text: String = format!($($fmt)*);
        unsafe {
            $crate::logger::invoke_reloaded_logger(
                text.as_ptr(), text.len(),
                $crate::logger::LogLevel::$ty,
            );
        }
    };
}


type LogFn = unsafe extern "C" fn(*const u8, usize, LogLevel) -> ();

/// A function pointer to invoke WriteAsync method in Reloaded-II's logger. This allows for
/// us to write into the console output and have that saved into a log file.
pub static RELOADED_LOGGER: OnceLock<LogFn> = OnceLock::new();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_reloaded_logger(cb: LogFn) {
    RELOADED_LOGGER.set(cb).unwrap();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn invoke_reloaded_logger(p: *const u8, len: usize, level: LogLevel) {
    unsafe { RELOADED_LOGGER.get().unwrap()(p, len, level) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn invoke_println(p: *const u8, len: usize, _: LogLevel) {
    println!("{}", unsafe { str::from_utf8_unchecked(std::slice::from_raw_parts(p, len)) });
}