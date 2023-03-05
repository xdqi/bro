use crate::browser::*;
use crate::types::*;
use crate::*;

// for Mime::from_str
use std::ffi::OsString;
use std::str::FromStr;

pub fn current_default_browser() -> Result<String> {
    // get from handlr
    let apps = &handlr_regex::apps::APPS;
    let handler = apps.get_handler(&mime::Mime::from_str("x-scheme-handler/https")?)?;

    // returns the .desktop file name
    Ok(handler.to_string())
}

pub fn set_default_browser() -> Result<()> {
    Ok(())
}

pub fn register() -> Result<()> {
    // create desktop file
    let content = format!(
        "[Desktop Entry]
Name=bro
GenericName=bro: a browser selector
Comment=Redirects open URLs to a browser of your choice.
Keywords=web;browser;internet;
Exec={} %u
StartupNotify=true
Terminal=false
Type=Application
#Icon=bro
Categories=Network;WebBrowser;
MimeType=x-scheme-handler/http;x-scheme-handler/https
",
        env::current_exe()?.to_str().expect("Invalid path")
    );
    let desktop_path = expand_path("~/.local/share/applications/bro.desktop")?;
    let mut file = fs::File::create(&desktop_path[0])?;
    file.write_all(content.as_bytes())?;

    // set handler
    let handler = handlr_regex::Handler::resolve(OsString::from("bro.desktop"))?;
    let mut apps = (*handlr_regex::APPS).clone();
    apps.set_handler(
        mime::Mime::from_str("x-scheme-handler/https")?,
        handler.clone(),
    );
    apps.set_handler(mime::Mime::from_str("x-scheme-handler/http")?, handler);
    apps.save()?;

    Ok(())
}

pub fn unregister() -> Result<()> {
    // remove desktop file
    let desktop_path = expand_path("~/.local/share/applications/bro.desktop")?;
    fs::remove_file(desktop_path[0].clone())?;
    Ok(())
}

pub fn available_browsers() -> Result<Vec<Browser>> {
    // list using handlr
    // filter browsers from applications that can handle http and https
    let mut browsers: Vec<Browser> = handlr_regex::apps::SystemApps::get_entries()?
        .map(|(_, e)| e)
        .filter(|e| {
            e.mimes
                .contains(&mime::Mime::from_str("x-scheme-handler/https").unwrap())
                || e.mimes
                    .contains(&mime::Mime::from_str("x-scheme-handler/http").unwrap())
        })
        .map(|e| Browser {
            // polish the content
            id: e.file_name.into_string().expect("File name is invalid"),
            name: e.name.clone(),
            command: e.exec.clone(),
            profiles: Vec::new(),
        })
        .collect();
    browsers.iter_mut().for_each(|b| {
        get_profiles(b).unwrap();
    });
    Ok(browsers)
}

pub fn expand_path(path: &str) -> Result<Vec<String>> {
    // expand HOME directory
    if path.contains("~") {
        let expanded = path.replace("~", &env::var("HOME")?);
        return Ok(vec![expanded]);
    }
    Ok(vec![path.to_string()])
}

pub fn detect_path(browser: &Browser, hint: &ProfileHint) -> Result<String> {
    if browser.id.as_str() == hint.exe_path {
        return Ok(browser
            .command
            .replace("%u", "") // remove one-URL placeholder
            .replace("%U", "") // remove URLs placeholder
            .trim_end() // remove trailing spaces
            .to_string());
    }
    Ok(String::new())
}
