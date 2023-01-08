use crate::utils::*;
use crate::*;

pub fn current_default_browser() -> Result<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let cur_assoc = hkcu.open_subkey(
        "SOFTWARE\\Microsoft\\Windows\\Shell\\Associations\\UrlAssociations\\https\\UserChoice",
    )?;
    Ok(cur_assoc.get_value("ProgId")?)
}

pub fn set_default_browser() -> Result<()> {
    // TODO: show a window
    spawn_shell_command(
        "control",
        &vec![
            String::from("/name"),
            String::from("Microsoft.DefaultPrograms"),
            String::from("/page"),
            String::from("pageDefaultProgram"),
        ],
    )?;
    Ok(())
}
