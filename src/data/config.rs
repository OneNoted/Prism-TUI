use crate::error::{PrismError, Result};
use configparser::ini::Ini;
use std::env;
use std::path::{Path, PathBuf};

pub struct PrismConfig {
    pub data_dir: PathBuf,
    instances_dir_override: Option<PathBuf>,
    #[allow(dead_code)]
    pub selected_instance: Option<String>,
}

impl PrismConfig {
    pub fn load(data_dir: &Path, instances_dir_override: Option<PathBuf>) -> Result<Self> {
        let config_path = data_dir.join("prismlauncher.cfg");
        let mut config = Ini::new();

        let selected_instance = if config_path.exists() {
            config
                .load(&config_path)
                .map_err(|e| PrismError::Config(e.to_string()))?;
            config.get("General", "SelectedInstance")
        } else {
            None
        };

        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            instances_dir_override,
            selected_instance,
        })
    }

    pub fn instances_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.instances_dir_override {
            dir.clone()
        } else {
            self.data_dir.join("instances")
        }
    }

    pub fn accounts_path(&self) -> PathBuf {
        self.data_dir.join("accounts.json")
    }
}

pub fn find_prism_data_dir(config_data_dir: Option<PathBuf>) -> Result<PathBuf> {
    // 1. Check environment variable first
    if let Ok(path) = env::var("PRISMLAUNCHER_DATA") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
        return Err(PrismError::Config(format!(
            "PRISMLAUNCHER_DATA directory does not exist: {}",
            path.display()
        )));
    }

    // 2. Config file data_dir
    if let Some(path) = config_data_dir {
        if path.exists() {
            return Ok(path);
        }
        return Err(PrismError::Config(format!(
            "Configured data_dir does not exist: {}",
            path.display()
        )));
    }

    // 3. Standard platform location
    if let Some(data_dir) = dirs::data_dir() {
        let standard = data_dir.join("PrismLauncher");
        if standard.exists() {
            return Ok(standard);
        }
    }

    // 4. Flatpak location (Linux only)
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            let flatpak = home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher");
            if flatpak.exists() {
                return Ok(flatpak);
            }
        }
    }

    Err(PrismError::DataDirNotFound)
}
