/// Delegates to [`log::trace!`] when the `logging` feature is enabled; no-op otherwise.
#[cfg(feature = "logging")]
macro_rules! trace {
    ($($arg:tt)*) => { log::trace!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! trace {
    ($($arg:tt)*) => { () };
}

/// Delegates to [`log::warn!`] when the `logging` feature is enabled; no-op otherwise.
#[cfg(feature = "logging")]
macro_rules! warn {
    ($($arg:tt)*) => { log::warn!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! warn {
    ($($arg:tt)*) => { () };
}
