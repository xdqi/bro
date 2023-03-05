use crate::types::*;
use crate::*;

pub fn current_default_browser() -> Result<String> {
    Ok(String::new())
}

pub fn set_default_browser() -> Result<()> {
    // TODO: show a window
    Ok(())
}

pub fn register() -> Result<()> {
    Ok(())
}

pub fn unregister() -> Result<()> {
    Ok(())
}

pub fn available_browsers() -> Result<Vec<Browser>> {
    Ok(Vec::new())
}

pub fn expand_path(path: &str) -> Result<Vec<String>> {
    Ok(Vec::new())
}
