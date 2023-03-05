use crate::browser::*;
use crate::types::*;
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

pub fn register() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // create new key in HKCU\Software\Clients\StartMenuInternet
    let (key, _) = hkcu
        .create_subkey("Software\\Clients\\StartMenuInternet\\bro")
        .expect("Create new key failed");
    // set default value
    key.set_value("", &"bro").expect("Set default value failed");

    // create new key in HKCU\Software\Clients\StartMenuInternet\bro\shell\open\command
    let (cmd_key, _) = key
        .create_subkey("shell\\open\\command")
        .expect("Create command key failed");
    // set default value
    let exe_path = env::current_exe().expect("Get current exe path failed");
    let exe_str_full = exe_path.to_str().expect("Get exe path string failed");
    let exe_str;
    if exe_str_full.starts_with("\\\\?\\") {
        // remove \\?\ prefix
        exe_str = &exe_str_full[4..];
    } else {
        exe_str = exe_str_full;
    }
    let cmd_val = format!("\"{}\"", exe_str);
    cmd_key
        .set_value("", &cmd_val.as_str())
        .expect("Set command failed");

    // create new key in HKCU\Software\Clients\StartMenuInternet\bro\DefaultIcon
    let (icon_key, _) = key
        .create_subkey("DefaultIcon")
        .expect("Create icon key failed");
    let icon_val = format!("\"{}\",0", exe_str);
    icon_key
        .set_value("", &icon_val.as_str())
        .expect("Set default icon failed");

    // create new key in HKCU\Software\Clients\StartMenuInternet\bro\Capabilities
    let (cap_key, _) = key
        .create_subkey("Capabilities")
        .expect("Create capabilities key failed");
    // set values
    cap_key
        .set_value("ApplicationName", &"bro")
        .expect("Set ApplicationName failed");
    cap_key
        .set_value(
            "ApplicationDescription",
            &"Redirects open URLs to a browser of your choice.",
        )
        .expect("Set ApplicationDescription failed");
    cap_key
        .set_value("ApplicationIcon", &icon_val)
        .expect("Set ApplicationIcon failed");

    // create new key in HKCU\Software\Clients\StartMenuInternet\bro\Capabilities\URLAssociations
    let (url_key, _) = cap_key
        .create_subkey("URLAssociations")
        .expect("Create URLAssociations key failed");
    // set values
    url_key
        .set_value("https", &"BroHTTP")
        .expect("Set https failed");
    url_key
        .set_value("http", &"BroHTTP")
        .expect("Set http failed");

    // create new key in HKCR\BroHTTP
    let (bro_key, _) = RegKey::predef(HKEY_CLASSES_ROOT)
        .create_subkey("BroHTTP")
        .expect("Create BroHTTP key failed");
    // set default value
    bro_key
        .set_value("", &"bro HTTP(s) Protocol")
        .expect("Set default value failed");

    // create new key in HKCR\BroHTTP\shell\open\command
    let (bro_cmd_key, _) = bro_key
        .create_subkey("shell\\open\\command")
        .expect("Create BroHTTP command key failed");
    // set default value
    let bro_cmd_val = format!("\"{}\" %1", exe_str);
    bro_cmd_key
        .set_value("", &bro_cmd_val.as_str())
        .expect("Set BroHTTP command failed");

    // create new key in HKCR\BroHTTP\Application
    let (bro_app_key, _) = bro_key
        .create_subkey("Application")
        .expect("Create BroHTTP Application key failed");
    // set values
    bro_app_key
        .set_value("ApplicationName", &"bro")
        .expect("Set ApplicationName failed");
    bro_app_key
        .set_value(
            "ApplicationDescription",
            &"Redirects open URLs to a browser of your choice.",
        )
        .expect("Set ApplicationDescription failed");

    // open HKCU\Software\RegisteredApplications
    let (reg_app_key, _) = hkcu
        .create_subkey("Software\\RegisteredApplications")
        .expect("Open RegisteredApplications failed");
    // add bro to registered applications
    reg_app_key
        .set_value(
            "bro",
            &"Software\\Clients\\StartMenuInternet\\bro\\Capabilities",
        )
        .expect("Set bro failed");

    Ok(())
}

pub fn unregister() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // delete key in HKCU\Software\Clients\StartMenuInternet
    hkcu.delete_subkey_all("Software\\Clients\\StartMenuInternet\\bro")
        .expect("Delete key failed");
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    // delete key in HKCR\BroHTTP
    hkcr.delete_subkey_all("BroHTTP")
        .expect("Delete key failed");
    Ok(())
}

fn get_browser(key: &RegKey, name: &str) -> Result<Browser> {
    let subkey = key.open_subkey(name)?;
    let mut ret = Browser {
        id: String::from(name),
        name: subkey.get_value("")?,
        command: subkey.open_subkey("shell\\open\\command")?.get_value("")?,
        profiles: Vec::new(),
    };
    let profiles = get_profiles(&mut ret)?;
    ret.profiles = profiles;
    Ok(ret)
}

pub fn available_browsers() -> Result<Vec<Browser>> {
    let mut ret: Vec<Browser> = Vec::new();
    for hkey in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let regkey = RegKey::predef(hkey);
        let browsers = regkey.open_subkey("Software\\Clients\\StartMenuInternet")?;
        for browser in browsers.enum_keys().map(|x| x.unwrap()) {
            let browser = get_browser(&browsers, &browser)?;
            // println!("{:?}", browser);
            ret.push(browser);
        }
    }

    Ok(ret)
}

pub fn expand_path(path: &str) -> Result<Vec<String>> {
    if path.contains("%programfiles%") {
        // detect binary program files that's same architecture with this binary
        let mut ret: Vec<String> = vec![path.replace("%programfiles%", &env::var("programfiles")?)];
        if is_wow64() {
            // detect amd64 binary under wow64 environment
            ret.push(path.replace("%programfiles%", &env::var("programw6432")?));
        } else if is_64() {
            // detect i386 binary under amd64 windows
            ret.push(path.replace("%programfiles%", &env::var("programfiles(x86)")?));
        }
        return Ok(ret);
    } else if path.contains("%appdata%") {
        return Ok(vec![path.replace("%appdata%", &env::var("appdata")?)]);
    } else if path.contains("%localappdata%") {
        return Ok(vec![
            path.replace("%localappdata%", &env::var("localappdata")?)
        ]);
    }
    Ok(vec![path.to_string()])
}

pub fn is_wow64() -> bool {
    let mut ret: i32 = 0;
    unsafe {
        let proc_handle = winapi::um::processthreadsapi::GetCurrentProcess();
        winapi::um::wow64apiset::IsWow64Process(proc_handle, &mut ret);
    }
    ret != 0
}
