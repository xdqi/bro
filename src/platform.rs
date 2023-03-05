#[cfg(windows)]
pub use crate::windows::*;

#[cfg(all(unix, not(target_os = "macos")))]
pub use crate::unix::*;
