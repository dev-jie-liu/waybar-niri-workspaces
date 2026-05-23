use regex::Regex;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub only_current_workspace: bool,
    #[serde(default = "default_icon_size")]
    pub icon_size: i32,
    #[serde(default)]
    pub ignore_rules: Vec<IgnoreRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IgnoreRule {
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "parse_optional_regex")]
    pub title_regex: Option<Regex>,
    #[serde(default)]
    pub title_contains: Option<String>,
    #[serde(default)]
    pub workspace: Option<u64>,
}

impl Settings {
    pub fn should_ignore(&self, app_id: Option<&str>, title: Option<&str>, workspace_id: Option<u64>) -> bool {
        for rule in &self.ignore_rules {
            let app_match = rule.app_id.as_ref().map_or(true, |id| app_id == Some(id.as_str()));
            let title_match = rule.title.as_ref().map_or(true, |t| title == Some(t.as_str()));
            let title_contains_match = rule.title_contains.as_ref().map_or(true, |contains| {
                title.map_or(false, |t| t.contains(contains))
            });
            let title_regex_match = rule.title_regex.as_ref().map_or(true, |regex| {
                title.map_or(false, |t| regex.is_match(t))
            });
            let workspace_match = rule.workspace.map_or(true, |ws| workspace_id == Some(ws));

            if app_match && title_match && title_contains_match && title_regex_match && workspace_match {
                return true;
            }
        }
        false
    }

    pub fn only_current_workspace(&self) -> bool {
        self.only_current_workspace
    }

    pub fn icon_size(&self) -> i32 {
        self.icon_size
    }
}

fn parse_optional_regex<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    let pattern: Option<String> = Option::deserialize(deserializer)?;
    pattern.map(|p| Regex::new(&p).map_err(serde::de::Error::custom)).transpose()
}

fn default_icon_size() -> i32 { 24 }
