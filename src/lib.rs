pub mod browser;
pub mod platform;
pub mod rule;
pub mod types;
#[cfg(all(unix, not(target_os = "macos")))]
pub mod unix;
pub mod utils;
#[cfg(windows)]
pub mod windows;

pub use std::env;
pub use std::fs;
pub use std::io::prelude::*;
pub use std::path::PathBuf;
pub use std::process::Command;

pub use anyhow::{Error, Ok, Result};
pub use regex::Regex;
pub use serde::{Deserialize, Serialize};
pub use wildmatch::WildMatch;
#[cfg(windows)]
pub use winreg::enums::*;
#[cfg(windows)]
pub use winreg::RegKey;
