use crate::types::*;
use crate::*;

impl Rule {
    pub fn new(matcher: &str, pattern: &str, browser: &str) -> Rule {
        Rule {
            matcher: String::from(matcher),
            pattern: String::from(pattern),
            browser: String::from(browser),
        }
    }
}

fn compile_rule(rule: &Rule) -> Result<CompiledRule> {
    match rule.matcher.as_str() {
        "WILDCARD" => Ok(CompiledRule {
            matcher: CompiledMatcher::Wildcard(WildMatch::new(&rule.pattern)),
            browser: rule.browser.clone(),
        }),
        "REGEX" => Ok(CompiledRule {
            matcher: CompiledMatcher::Regex(Regex::new(&rule.pattern)?),
            browser: rule.browser.clone(),
        }),
        "DOMAIN-WILDCARD" => Ok(CompiledRule {
            matcher: CompiledMatcher::Wildcard(WildMatch::new(&format!(
                "http?://{}/*",
                rule.pattern
            ))),
            browser: rule.browser.clone(),
        }),
        "DOMAIN" => Ok(CompiledRule {
            matcher: CompiledMatcher::Regex(Regex::new(&format!(
                r"http[s]?://{}/.*",
                rule.pattern.replace(".", r"\.")
            ))?),
            browser: rule.browser.clone(),
        }),
        "DOMAIN-SUFFIX" => Ok(CompiledRule {
            matcher: CompiledMatcher::Regex(Regex::new(&format!(
                r"http[s]?://(.+\.)?{}/.*",
                rule.pattern.replace(".", r"\.")
            ))?),
            browser: rule.browser.clone(),
        }),
        "FINAL" => Ok(CompiledRule {
            matcher: CompiledMatcher::Wildcard(WildMatch::new("*")),
            browser: rule.browser.clone(),
        }),
        other => return Err(Error::msg(format!("Unknown rule type {}", other))),
    }
}

pub fn compile_rules(rules: &Vec<Rule>) -> Result<Vec<CompiledRule>> {
    let mut ret: Vec<CompiledRule> = Vec::new();
    for rule in rules {
        ret.push(compile_rule(rule)?);
    }
    Ok(ret)
}

pub fn match_rules(rules: &Vec<CompiledRule>, uri: &str) -> Result<String> {
    for rule in rules {
        match &rule.matcher {
            CompiledMatcher::Wildcard(w) => {
                if w.matches(uri) {
                    return Ok(rule.browser.clone());
                }
            }
            CompiledMatcher::Regex(r) => {
                if r.is_match(uri) {
                    return Ok(rule.browser.clone());
                }
            }
        }
    }
    Ok(String::new()) // fallback to default rule
}
