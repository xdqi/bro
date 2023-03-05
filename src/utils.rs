use crate::*;

pub fn get_config_path() -> Result<PathBuf> {
    let mut exe_path = env::current_exe()?;
    exe_path.pop();
    exe_path.push("bro.json");
    Ok(exe_path)
}

pub fn spawn_shell_command(exe: &str, args: &Vec<String>) -> Result<()> {
    Command::new(exe).args(args).spawn()?;
    Ok(())
}

#[cfg(target_pointer_width = "32")]
pub fn is_64() -> bool {
    return false;
}
#[cfg(target_pointer_width = "64")]
pub fn is_64() -> bool {
    return true;
}
