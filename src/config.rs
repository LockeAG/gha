use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: Option<String>,
    pub interval: Option<u64>,
    pub days: Option<u64>,
    pub per_page: Option<u8>,
    pub max_repos: Option<usize>,
    pub orgs: Vec<String>,
    pub repos: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        });
    base.join("gha").join("config.toml")
}

pub fn generate_sample() -> &'static str {
    r#"# gha configuration
# CLI flags override these values.
# Path: ~/.config/gha/config.toml

# Color theme: catppuccin-mocha, tokyo-night, tokyo-night-storm
# theme = "catppuccin-mocha"

# Poll interval in seconds (min 10)
# interval = 30

# Only watch repos active in last N days (0 = all)
# days = 7

# Max workflow runs fetched per repo
# per_page = 20

# Max repos to auto-watch from orgs (0 = all, press 'a' for more)
# max_repos = 5

# Organizations to watch
# orgs = ["MyOrg"]

# Specific repos to always watch (not filtered by --days)
# repos = ["owner/repo"]
"#
}
