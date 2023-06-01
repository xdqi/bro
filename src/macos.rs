/// macOS implementation
/// ref: https://chromium.googlesource.com/chromium/src/+/master/chrome/browser/shell_integration_mac.mm
use crate::browser::*;
use crate::types::*;
use crate::*;
use block::ConcreteBlock;
use cocoa_foundation::{base::*, foundation::*};
use fruitbasket::{FruitApp, FruitCallbackKey, RunPeriod};
use objc::runtime::Object;
use std::path::PathBuf;
use std::result::Result::Ok;

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

static mut FOUND_URL: Option<String> = None;

pub fn macos_init() -> Option<String> {
    let mut app = FruitApp::new();
    // Register a callback for when the ObjC application finishes launching
    let stopper = app.stopper();
    app.register_callback(
        FruitCallbackKey::Method("applicationWillFinishLaunching:"),
        Box::new(move |_event| {
            println!("applicationDidFinishLaunching.");
            stopper.stop();
        }),
    );

    // Run until callback is called
    println!("Spawned process started.  Run until applicationDidFinishLaunching.");
    let _ = app.run(RunPeriod::Forever);

    println!("Application launched.  Registering URL callbacks.");
    // Register a callback to get receive custom URL schemes from any Mac program
    app.register_apple_event(fruitbasket::kInternetEventClass, fruitbasket::kAEGetURL);
    let stopper = app.stopper();
    app.register_callback(
        FruitCallbackKey::Method("handleEvent:withReplyEvent:"),
        Box::new(move |event| {
            // Event is a raw NSAppleEventDescriptor.
            // Fruitbasket has a parser for URLs.  Call that to get the URL:
            let url: String = fruitbasket::parse_url_event(event);
            println!("Received URL: {}", url);
            unsafe {
                FOUND_URL = Some(url);
            }
            stopper.stop();
        }),
    );

    // let stopper = app.stopper();
    // app.register_callback(
    //     FruitCallbackKey::Method("application:openFile:"),
    //     Box::new(move |file| {
    //         // File is a raw NSString.
    //         // Fruitbasket has a converter to Rust String:
    //         let file: String = fruitbasket::nsstring_to_string(file);
    //         println!("Received file: {}", file);
    //         stopper.stop();
    //     }),
    // );

    // Run 'forever', until one of the URL or file callbacks fire
    println!("Spawned process running!");
    let _ = app.run(RunPeriod::Forever);
    println!("Run loop stopped after URL callback.");

    // Cleanly terminate
    // fruitbasket::FruitApp::terminate(0);
    // println!("This will not print.");
    unsafe { FOUND_URL.clone() }
}

pub fn current_default_browser() -> Result<String> {
    unsafe {
        // returns the app name
        let nsworkspace = class!(NSWorkspace);
        let shared_workspace: *mut Object = msg_send![nsworkspace, sharedWorkspace];
        let url_str = NSString::alloc(nil).init_str("https://www.google.com");
        let url = NSURL::URLWithString_(nil, url_str);
        let app_url: *mut Object = msg_send![shared_workspace, URLForApplicationToOpenURL: url];

        let fs_rep = msg_send![app_url, fileSystemRepresentation];
        // returns the app full path (e.g. /Applications/Google Chrome.app, not just Google Chrome)
        let ret = std::ffi::CStr::from_ptr(fs_rep)
            .to_str()
            .unwrap()
            .to_string();

        // TODO: release ObjC memory
        Ok(ret)

        // another way to return Google Chrome
        // let app_url_path = app_url.path();
        // let nsfilemanager = class!(NSFileManager);
        // let default_manager: *mut Object = msg_send![nsfilemanager, defaultManager];
        // let app_name: *mut Object = msg_send![default_manager, displayNameAtPath:app_url_path];
        // let ret = std::ffi::CStr::from_ptr(app_name.UTF8String()).to_str().unwrap().to_string();
    }
}

pub fn set_default_browser() -> Result<()> {
    // macOS 12+ implements a new way to set as default browser
    unsafe {
        let nsworkspace = class!(NSWorkspace);
        let shared_workspace: *mut Object = msg_send![nsworkspace, sharedWorkspace];

        let bundle = NSURL::initWithString_(
            nil,
            NSString::alloc(nil).init_str(
                // TODO: set to bro later
                "file:///Applications/Firefox.app",
            ),
        );
        let completion_handler = ConcreteBlock::new(|_: *mut Object| {});
        let http = NSString::alloc(nil).init_str("http");
        let https = NSString::alloc(nil).init_str("https");
        let _: () = msg_send![shared_workspace, setDefaultApplicationAtURL:bundle toOpenURLsWithScheme:http completionHandler:completion_handler.clone()];
        let _: () = msg_send![shared_workspace, setDefaultApplicationAtURL:bundle toOpenURLsWithScheme:https completionHandler:completion_handler.clone()];
        Ok(())
        // TODO: put the confirm window onto front
    }
}

pub fn register() -> Result<()> {
    Ok(())
}

pub fn unregister() -> Result<()> {
    Ok(())
}

pub fn available_browsers() -> Result<Vec<Browser>> {
    // get from Info.plist in every app in /Applications that can handle http and https
    let mut app_path = vec![PathBuf::from("/Applications")];
    let mut user_apps = PathBuf::from(env::var("HOME")?);
    user_apps.push("Applications");
    app_path.push(user_apps);

    let mut browsers = Vec::<Browser>::new();
    for path in app_path.iter() {
        let dir = std::fs::read_dir(path);
        if dir.is_err() {
            println!("Error listing dir {:?}: {:?}", app_path, dir.err());
            continue;
        }
        // for every app in /Applications & ~/Applications
        for maybe_app in dir.unwrap() {
            if maybe_app.is_err() {
                println!("Error getting item {:?}: {:?}", app_path, maybe_app.err());
                continue;
            }
            let app = maybe_app.unwrap();
            // App must be a directory and end with .app
            if !app.file_type().unwrap().is_dir()
                || !app.file_name().to_str().unwrap().ends_with(".app")
            {
                continue;
            }
            let mut plist_path = app.path();
            plist_path.push("Contents");
            plist_path.push("Info.plist");
            // App must have an Info.plist
            if !plist_path.exists() {
                continue;
            }
            // App must be able to handle http or https
            if unsafe { !app_can_handle_http_or_https(&plist_path) } {
                continue;
            }

            browsers.push(Browser {
                id: app.file_name().into_string().unwrap(),
                name: app.file_name().into_string().unwrap(),
                command: app.path().into_os_string().into_string().unwrap(),
                profiles: vec![],
            });
        }
    }
    browsers.iter_mut().for_each(|b| {
        get_profiles(b).unwrap();
    });
    Ok(browsers)
}

unsafe fn app_can_handle_http_or_https(path: &PathBuf) -> bool {
    let plist_path = NSString::alloc(nil).init_str(path.as_os_str().to_str().unwrap());
    let dict = NSDictionary::dictionaryWithContentsOfFile_(nil, plist_path);
    if dict.is_null() {
        return false;
    }
    let types_str = NSString::alloc(nil).init_str("CFBundleURLTypes");
    let types = dict.objectForKey_(types_str);
    if types.is_null() {
        return false;
    }

    let schemes_str = NSString::alloc(nil).init_str("CFBundleURLSchemes");
    for i in 0u64..NSDictionary::count(types) {
        let type_ = types.objectAtIndex(i);
        let schemes = type_.objectForKey_(schemes_str);
        if schemes.is_null() {
            continue;
        }
        for j in 0u64..NSDictionary::count(schemes) {
            let scheme = schemes.objectAtIndex(j);
            let scheme = scheme.UTF8String();
            let scheme = std::ffi::CStr::from_ptr(scheme).to_str().unwrap();
            if scheme == "http" || scheme == "https" {
                return true;
            }
        }
    }
    false
    // TODO: release these ObjC memory
    // let _: () = objc::msg_send![schemes_str, release];
    // let _: () = objc::msg_send![types_str, release];
    // let _: () = objc::msg_send![dict, release];
    // let _: () = objc::msg_send![plist_path, release];
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
        return Ok(browser.command.clone());
    }
    Ok(String::new())
}
