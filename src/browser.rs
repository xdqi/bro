use crate::platform::*;
use crate::types::*;
use crate::*;

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
        #[cfg(windows)]
        exe_path: "%programfiles%\\Google\\Chrome\\Application\\chrome.exe",
        #[cfg(windows)]
        profiles_path: "%localappdata%\\Google\\Chrome\\User Data",
        #[cfg(all(unix, not(target_os = "macos")))]
        exe_path: "google-chrome.desktop",
        #[cfg(all(unix, not(target_os = "macos")))]
        profiles_path: "~/.config/google-chrome",
        #[cfg(target_os = "macos")]
        exe_path: "Google Chrome.app",
        #[cfg(target_os = "macos")]
        profiles_path: "~/Library/Application Support/Google/Chrome",

        private_arg: "--incognito",
        private_name: "Incognito mode",
        profile_arg: "--profile-directory=",
        profile_arg_ctor: construct_chrome_profile_arguments,
        icon_path: "Google Profile.ico",
        detector: check_chrome_profile,
    },
    ProfileHint {
        #[cfg(windows)]
        exe_path: "%programfiles%\\Google\\Chrome Beta\\Application\\chrome.exe",
        #[cfg(windows)]
        profiles_path: "%localappdata%\\Google\\Chrome Beta\\User Data",
        #[cfg(all(unix, not(target_os = "macos")))]
        exe_path: "google-chrome-beta.desktop",
        #[cfg(all(unix, not(target_os = "macos")))]
        profiles_path: "~/.config/google-chrome-beta",
        #[cfg(target_os = "macos")]
        exe_path: "Google Chrome Beta.app",
        #[cfg(target_os = "macos")]
        profiles_path: "~/Library/Application Support/Google/Chrome Beta",

        private_arg: "--incognito",
        private_name: "Incognito mode",
        profile_arg: "--profile-directory=",
        profile_arg_ctor: construct_chrome_profile_arguments,
        icon_path: "Google Profile.ico",
        detector: check_chrome_profile,
    },
    ProfileHint {
        #[cfg(windows)]
        exe_path: "%programfiles%\\Microsoft\\Edge\\Application\\msedge.exe",
        #[cfg(windows)]
        profiles_path: "%localappdata%\\Microsoft\\Edge\\User Data",
        #[cfg(all(unix, not(target_os = "macos")))]
        exe_path: "microsoft-edge.desktop",
        #[cfg(all(unix, not(target_os = "macos")))]
        profiles_path: "~/.config/microsoft-edge",
        #[cfg(target_os = "macos")]
        exe_path: "Microsoft Edge.app",
        #[cfg(target_os = "macos")]
        profiles_path: "~/Library/Application Support/Microsoft Edge",

        private_arg: "-inprivate",
        private_name: "InPrivate mode",
        profile_arg: "--profile-directory=",
        profile_arg_ctor: construct_chrome_profile_arguments,
        icon_path: "Edge Profile.ico",
        detector: check_chrome_profile,
    },
    ProfileHint {
        #[cfg(windows)]
        exe_path: "%programfiles%\\Mozilla Firefox\\firefox.exe",
        #[cfg(windows)]
        profiles_path: "%appdata%\\Mozilla\\Firefox\\Profiles",
        #[cfg(all(unix, not(target_os = "macos")))]
        exe_path: "firefox.desktop",
        #[cfg(all(unix, not(target_os = "macos")))]
        profiles_path: "~/.mozilla/firefox",
        #[cfg(target_os = "macos")]
        exe_path: "Firefox.app",
        #[cfg(target_os = "macos")]
        profiles_path: "~/Library/Application Support/Firefox/Profiles",

        private_arg: "-private-window",
        private_name: "Private Browsing",
        profile_arg_ctor: construct_firefox_profile_arguments,
        profile_arg: "-P",
        icon_path: "",
        detector: check_firefox_profile,
    },
];

pub fn get_profiles(browser: &mut Browser) -> Result<()> {
    let mut ret: Vec<Profile> = Vec::new();
    for hint in PROFILE_HINTS {
        // only detect profile when startup command(Windows), desktop file name(Linux) matches
        // and profiles directory exists
        let detected_path = detect_path(&browser, &hint)?;
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
        browser.profiles = ret;
        return Ok(());
    }
    Ok(())
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
            #[cfg(target_os = "macos")]
            return Ok((
                String::from("open"),
                vec![
                    String::from("-n"),      // launch a new instance
                    String::from("-a"),      // using the application
                    browser.command.clone(), // application name
                    String::from("--args"),  // pass arguments
                    String::from(uri),       // uri
                ],
            ));
            #[cfg(not(target_os = "macos"))]
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

            #[cfg(target_os = "macos")]
            {
                let mut vec = vec![
                    String::from("-n"),
                    String::from("-a"),
                    browser.command.clone(),
                    String::from("--args"),
                ];
                vec.extend(profile.args.clone());
                vec.push(uri.to_string());
                return Ok((String::from("open"), vec));
            }
            #[cfg(not(target_os = "macos"))]
            {
                let mut vec = profile.args.clone();
                vec.push(String::from(uri));
                // matched profile
                return Ok((String::from(browser.command.clone()), vec));
            }
        }

        // when no profile is found
        return Err(Error::msg(format!(
            "Unknown profile {} for browser {}",
            vec[1], vec[0]
        )));
    }
    return Err(Error::msg(format!("Unkown browser {}", vec[0])));
}
