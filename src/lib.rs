pub mod browser;
pub mod rule;
pub mod system;
pub mod utils;

pub use std::env;
pub use std::fs;
pub use std::io::prelude::*;
pub use std::path::PathBuf;
pub use std::process::Command;

extern crate serde_json;
extern crate winapi;
extern crate winreg;

pub use anyhow::{Error, Ok, Result};
pub use regex::Regex;
pub use serde::{Deserialize, Serialize};
pub use wildmatch::WildMatch;
pub use winreg::enums::*;
pub use winreg::RegKey;
