use crate::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub args: Vec<String>,
    pub icon_path: String,
}

pub struct ProfileHint {
    pub exe_path: &'static str,
    pub profiles_path: &'static str,
    pub private_arg: &'static str,
    pub private_name: &'static str,
    pub profile_arg: &'static str,
    pub profile_arg_ctor: fn(&ProfileHint, &str) -> Vec<String>,
    pub icon_path: &'static str,
    pub detector: fn(&str, &str) -> Result<bool>,
}

impl ProfileHint {
    pub fn construct_profile_arguments(self: &ProfileHint, name: &str) -> Vec<String> {
        (self.profile_arg_ctor)(self, name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Browser {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub profiles: Vec<Profile>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    pub detected_browsers: Vec<Browser>,
    pub custom_browsers: Vec<Browser>,
    pub rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub matcher: String,
    #[serde(skip_serializing_if = "String::is_empty", default = "String::new")]
    pub pattern: String,
    pub browser: String,
}

pub enum CompiledMatcher {
    Wildcard(WildMatch),
    Regex(Regex),
}
pub struct CompiledRule {
    pub matcher: CompiledMatcher,
    pub browser: String,
}
