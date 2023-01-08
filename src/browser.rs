use crate::utils::*;
use crate::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    id: String,
    name: String,
    args: Vec<String>,
    icon_path: String,
}

pub struct ProfileHint {
    exe_path: &'static str,
    profiles_path: &'static str,
    private_arg: &'static str,
    private_name: &'static str,
    profile_arg: &'static str,
    profile_arg_ctor: fn(&ProfileHint, &str) -> Vec<String>,
    icon_path: &'static str,
    detector: fn(&str, &str) -> Result<bool>,
}

impl ProfileHint {
    fn construct_profile_arguments(self: &ProfileHint, name: &str) -> Vec<String> {
        (self.profile_arg_ctor)(self, name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Browser {
    id: String,
    name: String,
    command: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    profiles: Vec<Profile>,
}

fn check_chrome_profile(profiles_path: &str, profile_name: &str) -> Result<bool> {
    let mut path = PathBuf::from(profiles_path);
    path.extend(&[profile_name, "History"]);

    Ok(path.exists() && fs::metadata(path)?.is_file())
}

fn check_firefox_profile(profiles_path: &str, profile_name: &str) -> Result<bool> {
    let mut path = PathBuf::from(profiles_path);
    path.extend(&[profile_name, "cookies.sqlite"]);

    Ok(path.exists() && fs::metadata(path)?.is_file())
}

fn construct_chrome_profile_arguments(hint: &ProfileHint, name: &str) -> Vec<String> {
    return vec![format!("{}{}", hint.profile_arg, name)];
}

fn construct_firefox_profile_arguments(hint: &ProfileHint, name: &str) -> Vec<String> {
    return vec![String::from(hint.profile_arg), String::from(name)];
}

const PROFILE_HINTS: &'static [ProfileHint] = &[
    ProfileHint {
        exe_path: "%programfiles%\\Google\\Chrome\\Application\\chrome.exe",
        profiles_path: "%localappdata%\\Google\\Chrome\\User Data",
        private_arg: "--incognito",
        private_name: "Incognito mode",
        profile_arg: "--profile-directory=",
        profile_arg_ctor: construct_chrome_profile_arguments,
        icon_path: "Google Profile.ico",
        detector: check_chrome_profile,
    },
    ProfileHint {
        exe_path: "%programfiles%\\Google\\Chrome Beta\\Application\\chrome.exe",
        profiles_path: "%localappdata%\\Google\\Chrome Beta\\User Data",
        private_arg: "--incognito",
        private_name: "Incognito mode",
        profile_arg: "--profile-directory=",
        profile_arg_ctor: construct_chrome_profile_arguments,
        icon_path: "Google Profile.ico",
        detector: check_chrome_profile,
    },
    ProfileHint {
        exe_path: "%programfiles%\\Microsoft\\Edge\\Application\\msedge.exe",
        profiles_path: "%localappdata%\\Microsoft\\Edge\\User Data",
        private_arg: "-inprivate",
        private_name: "InPrivate mode",
        profile_arg: "--profile-directory=",
        profile_arg_ctor: construct_chrome_profile_arguments,
        icon_path: "Edge Profile.ico",
        detector: check_chrome_profile,
    },
    ProfileHint {
        exe_path: "%programfiles%\\Mozilla Firefox\\firefox.exe",
        profiles_path: "%appdata%\\Mozilla\\Firefox\\Profiles",
        private_arg: "-private-window",
        private_name: "Private Browsing",
        profile_arg_ctor: construct_firefox_profile_arguments,
        profile_arg: "-P",
        icon_path: "",
        detector: check_firefox_profile,
    },
];

fn expand_path(path: &str) -> Result<Vec<String>> {
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

fn detect_path(command: &str, paths: &Vec<String>) -> String {
    for path in paths {
        if command.contains(path) {
            return path.clone();
        }
    }
    return String::new();
}

pub fn get_profiles(browser: &mut Browser) -> Result<Vec<Profile>> {
    let mut ret: Vec<Profile> = Vec::new();
    for hint in PROFILE_HINTS {
        let paths = expand_path(hint.exe_path)?;
        // only detect profile when startup command matches and profile
        let detected_path = detect_path(&browser.command, &paths);
        if detected_path.is_empty() || hint.profiles_path.is_empty() {
            continue;
        }
        // change browser command to path of executable
        browser.command = detected_path;
        let profile_paths = expand_path(hint.profiles_path)?;
        assert!(profile_paths.len() == 1);
        ret.push(Profile {
            id: String::from("__PRIVATE__"),
            name: String::from(hint.private_name),
            args: vec![String::from(hint.private_arg)],
            icon_path: String::new(),
        });
        let profile_path = &profile_paths[0];
        let dirs = std::fs::read_dir(profile_path)?;
        for dir in dirs {
            let dir_entry = dir?;
            let dir_name = dir_entry.file_name();
            let dir_name_string = dir_name.clone().into_string().unwrap();
            // process only directory
            if !dir_entry.metadata()?.is_dir()
                || !(hint.detector)(profile_path, dir_name.to_str().unwrap())?
            {
                continue;
            }
            let mut icon_path = String::new();
            if !hint.icon_path.is_empty() {
                let mut path = PathBuf::from(profile_path);
                path.extend(&[dir_name_string.clone(), hint.icon_path.to_string()]);
                // println!("{:?}", path);
                if path.exists() {
                    icon_path = path.to_str().unwrap().to_string();
                }
            }
            ret.push(Profile {
                id: dir_name_string.clone(),
                name: dir_name_string.clone(),
                args: hint.construct_profile_arguments(&dir_name_string),
                icon_path: icon_path,
            });
        }
        // println!("{} {:?} {:?} {:?}", browser.command, paths, browser, ret);
        return Ok(ret);
    }
    Ok(ret)
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

pub fn launch_browser_command(
    browsers: &Vec<Browser>,
    browser_spec: &String,
    uri: &str,
) -> Result<(String, Vec<String>)> {
    let vec: Vec<&str> = browser_spec.split(":").collect();
    if vec.len() > 2 {
        return Err(Error::msg(format!(
            "Unkown browser specification {}",
            browser_spec
        )));
    }

    for browser in browsers {
        if browser.id != vec[0] {
            continue;
        }
        // use browser itself with no profile
        if vec.len() == 1 {
            return Ok((
                String::from(browser.command.clone()),
                vec![String::from(uri)],
            ));
        }

        for profile in &browser.profiles {
            if profile.id != vec[1] {
                continue;
            }

            // construct arguments
            let mut vec = profile.args.clone();
            vec.push(String::from(uri));
            // matched profile
            return Ok((String::from(browser.command.clone()), vec));
        }

        // when no profile is found
        return Err(Error::msg(format!(
            "Unknown profile {} for browser {}",
            vec[1], vec[0]
        )));
    }
    return Err(Error::msg(format!("Unkown browser {}", vec[0])));
}
