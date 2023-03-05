use bro::browser::*;
use bro::platform::*;
use bro::rule::*;
use bro::types::*;
use bro::utils::*;
use bro::*;

#[derive(Serialize, Deserialize)]
struct Config {
    detected_browsers: Vec<Browser>,
    custom_browsers: Vec<Browser>,
    rules: Vec<Rule>,
}

fn open_uri(uri: &str) -> Result<()> {
    let contents = fs::read_to_string(get_config_path()?).expect("Config not found");
    let mut config: Config = serde_json::from_str(&contents)?;
    if config.detected_browsers.is_empty() || config.rules.is_empty() {
        return Err(Error::msg(format!("Invalid config file")));
    }

    let compiled = compile_rules(&config.rules).unwrap();
    let browser = match_rules(&compiled, uri).unwrap();

    // add custom browsers at back
    config.detected_browsers.extend(config.custom_browsers);
    let cmd = launch_browser_command(&config.detected_browsers, &browser, uri).unwrap();
    println!("{} uses {:?}", uri, cmd);
    spawn_shell_command(&cmd.0, &cmd.1)?;

    Ok(())
}

fn write_example_config() -> Result<()> {
    let config: Config = Config {
        detected_browsers: available_browsers().unwrap(),
        custom_browsers: vec![],
        rules: vec![
            Rule::new("DOMAIN-SUFFIX", "contoso.com", "Google Chrome:Profile 1"),
            Rule::new("FINAL", "", "Google Chrome:Default"),
        ],
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    let mut file = fs::File::create(get_config_path()?)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

const FORMAT_SPEC: &str = r#"Usage:

bro              Show this help message
bro --register   Register as default browser
bro --unregister Unregister as default browser
bro <url>        Open URL in your desired browser

bro.json specification:

detected_browsers section contains all browser that Bro has detected, this section must not be changed, otherwise all changes will be lost after new launch of Bro settings.

custom_browsers section contains all browser that's defined by user, the format is the same as detected_browsers.

rules contains rules for matching browser, rule are matched from top to down, and every rule can contain the following columns:
1. when matcher is WILDCARD or REGEX, matching pattern against the full URL (using wildcard or regular expression syntax)
e.g. WILDCARD pattern "*://*.google.*" will match "http://www.google.com/", "https://www.google.co.uk/", "https://wtf.google.another.site/", "https://another.site/some.google.pdf"
2. when matcher is DOMAIN, matching URL that domain is strictly the same as the pattern
e.g. DOMAIN pattern "google.com" will match "https://google.com/", won't match "https://www.google.com/" or "https://www.google.com.hk/"
3. when matcher is DOMAIN-SUFFIX, matching URL that is the pattern or a subdomain of that pattern
e.g. DOMAIN-SUFFIX pattern "google.com" will match "https://google.com/" and "https://drive.google.com/", but will not match "https://www.google.com.hk/"
4. when matcher is DOMAIN-WILDCARD, matching pattern against the domain name only
e.g. DOMAIN-WILDCARD "*.google.*" will match "http://www.google.com/", "https://www.google.co.uk/", "https://wtf.google.another.site/", but will not match "https://another.site/some.google.pdf"
5. when matcher is FINAL, pattern is ignored. It's the default rule that no rule above has matched the URL

browser syntax: <browser.id>:<profile.id> (specifying profile) or <browser.id> (just launch the browser)

"#;

fn main() {
    env_logger::init();
    // current_default_browser().unwrap();
    // set_default_browser().unwrap();

    // write_example_config().unwrap();

    // let uris = [
    //     "https://www.google.com/",
    //     "https://github.com/",
    // ];
    // for uri in uris.iter() {
    //     open_uri(uri).unwrap();
    // }
    let argv: Vec<String> = env::args().collect();
    if argv.len() == 2 {
        if argv[1] == "--register" {
            register().unwrap();
            set_default_browser().unwrap();
        } else if argv[1] == "--unregister" {
            unregister().unwrap();
        } else {
            open_uri(&argv[1]).unwrap();
        }
    } else {
        let config_name = get_config_path().unwrap();
        if !fs::metadata(&config_name).is_ok() {
            write_example_config().unwrap();
            println!(
                "Creating example config at {}",
                config_name.to_str().unwrap()
            );
        }
        println!("Usage: {} URL", argv[0]);
        println!("{}", FORMAT_SPEC);
    }
}
