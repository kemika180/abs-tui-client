use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use color_eyre::eyre::{Result, eyre};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub theme: String,
    pub mpd: MpdSettings,
    pub vim_motions: bool,
    pub step_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerSettings {
    pub url: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MpdSettings {
    pub address: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                url: None,
                username: None,
                token: None,
            },
            theme: "tokyo-night".to_string(),
            mpd: MpdSettings {
                address: "localhost:6600".to_string(),
            },
            vim_motions: true,
            step_seconds: 30,
        }
    }
}

pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "abs-tui", "abs-tui-client")
        .ok_or_else(|| eyre!("Could not determine project directories"))
}

pub fn load_settings() -> Result<Settings> {
    let dirs = get_project_dirs()?;
    let config_dir = dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        return Ok(Settings::default());
    }

    let settings = config::Config::builder()
        .add_source(config::File::from(config_path))
        .build()?;

    Ok(settings.try_deserialize()?)
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    let dirs = get_project_dirs()?;
    let config_dir = dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir)?;
    }

    let toml = toml::to_string(settings)?;
    std::fs::write(config_path, toml)?;

    Ok(())
}
