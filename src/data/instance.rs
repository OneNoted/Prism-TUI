use crate::error::{PrismError, Result};
use configparser::ini::Ini;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub group: Option<String>,
    pub minecraft_version: String,
    pub mod_loader: Option<String>,
    pub total_time_played: u64,
    pub last_launch: Option<i64>,
    pub server_join: Option<ServerJoin>,
}

#[derive(Debug, Clone)]
pub struct ServerJoin {
    pub enabled: bool,
    pub address: String,
}

#[derive(Deserialize)]
struct MmcPack {
    components: Vec<Component>,
}

#[derive(Deserialize)]
struct Component {
    uid: String,
    version: Option<String>,
    #[serde(rename = "cachedVersion")]
    cached_version: Option<String>,
}

impl Component {
    fn get_version(&self) -> Option<&str> {
        self.version.as_deref().or(self.cached_version.as_deref())
    }
}

/// Possible Minecraft folder names in PrismLauncher instances
const MINECRAFT_FOLDERS: &[&str] = &[".minecraft", "minecraft"];

impl Instance {
    /// Find the Minecraft folder within the instance directory
    pub fn minecraft_dir(&self) -> Option<PathBuf> {
        for folder in MINECRAFT_FOLDERS {
            let path = self.path.join(folder);
            if path.exists() && path.is_dir() {
                return Some(path);
            }
        }
        None
    }

    pub fn load(path: PathBuf, groups: &HashMap<String, String>) -> Result<Self> {
        let id = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PrismError::Other("Invalid instance path".into()))?
            .to_string();

        let config_path = path.join("instance.cfg");
        let mut config = Ini::new();

        let (name, total_time_played, last_launch, server_join) = if config_path.exists() {
            config
                .load(&config_path)
                .map_err(|e| PrismError::Config(e.to_string()))?;

            let name = config.get("General", "name").unwrap_or_else(|| id.clone());

            let total_time_played = config
                .get("General", "totalTimePlayed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let last_launch = config
                .get("General", "lastLaunchTime")
                .and_then(|s| s.parse().ok());

            let join_enabled = config
                .get("General", "JoinServerOnLaunch")
                .map(|s| s == "true")
                .unwrap_or(false);

            let join_address = config.get("General", "JoinServerOnLaunchAddress");

            let server_join = join_address.map(|address| ServerJoin {
                enabled: join_enabled,
                address,
            });

            (name, total_time_played, last_launch, server_join)
        } else {
            (id.clone(), 0, None, None)
        };

        let (minecraft_version, mod_loader) = parse_mmc_pack(&path)?;

        let group = groups.get(&id).cloned();

        Ok(Self {
            id,
            name,
            path,
            group,
            minecraft_version,
            mod_loader,
            total_time_played,
            last_launch,
            server_join,
        })
    }

    pub fn servers_dat_path(&self) -> PathBuf {
        self.minecraft_dir()
            .map(|d| d.join("servers.dat"))
            .unwrap_or_else(|| self.path.join(".minecraft/servers.dat"))
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.minecraft_dir()
            .map(|d| d.join("logs"))
            .unwrap_or_else(|| self.path.join(".minecraft/logs"))
    }

    pub fn formatted_playtime(&self) -> String {
        let hours = self.total_time_played / 3600;
        if hours > 0 {
            format!("{}h played", hours)
        } else {
            let minutes = self.total_time_played / 60;
            format!("{}m played", minutes)
        }
    }

    pub fn mods_count(&self) -> usize {
        self.minecraft_dir()
            .map(|d| d.join("mods"))
            .filter(|p| p.exists())
            .and_then(|p| std::fs::read_dir(p).ok())
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|ext| ext == "jar" || ext == "zip")
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn saves_count(&self) -> usize {
        self.minecraft_dir()
            .map(|d| d.join("saves"))
            .filter(|p| p.exists())
            .and_then(|p| std::fs::read_dir(p).ok())
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn resource_packs_count(&self) -> usize {
        self.minecraft_dir()
            .map(|d| d.join("resourcepacks"))
            .filter(|p| p.exists())
            .and_then(|p| std::fs::read_dir(p).ok())
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0)
    }

    pub fn formatted_last_launch(&self) -> String {
        match self.last_launch {
            Some(ts) if ts > 0 => {
                use chrono::{DateTime, Local, Utc};
                let dt = DateTime::<Utc>::from_timestamp(ts / 1000, 0);
                match dt {
                    Some(utc) => {
                        let local: DateTime<Local> = utc.into();
                        local.format("%Y-%m-%d %H:%M").to_string()
                    }
                    None => "Unknown".to_string(),
                }
            }
            _ => "Never".to_string(),
        }
    }

    pub fn formatted_playtime_full(&self) -> String {
        let total = self.total_time_played;
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    pub fn set_server_join(&mut self, enabled: bool, address: Option<String>) -> Result<()> {
        let config_path = self.path.join("instance.cfg");
        let mut config = Ini::new();

        if config_path.exists() {
            config
                .load(&config_path)
                .map_err(|e| PrismError::Config(e.to_string()))?;
        }

        config.set("General", "JoinServerOnLaunch", Some(enabled.to_string()));

        if let Some(addr) = &address {
            config.set("General", "JoinServerOnLaunchAddress", Some(addr.clone()));
        }

        config
            .write(&config_path)
            .map_err(|e| PrismError::Config(e.to_string()))?;

        self.server_join = address.map(|addr| ServerJoin {
            enabled,
            address: addr,
        });

        Ok(())
    }
}

fn parse_mmc_pack(instance_path: &Path) -> Result<(String, Option<String>)> {
    let pack_path = instance_path.join("mmc-pack.json");

    if !pack_path.exists() {
        return Ok(("Unknown".into(), None));
    }

    let content = fs::read_to_string(&pack_path)?;
    let pack: MmcPack = serde_json::from_str(&content)?;

    let mut minecraft_version = "Unknown".to_string();
    let mut mod_loader = None;

    for component in pack.components {
        match component.uid.as_str() {
            "net.minecraft" => {
                if let Some(ver) = component.get_version() {
                    minecraft_version = ver.to_string();
                }
            }
            "net.minecraftforge" => mod_loader = Some("Forge".to_string()),
            "net.fabricmc.fabric-loader" => mod_loader = Some("Fabric".to_string()),
            "org.quiltmc.quilt-loader" => mod_loader = Some("Quilt".to_string()),
            "net.neoforged" => mod_loader = Some("NeoForge".to_string()),
            _ => {}
        }
    }

    Ok((minecraft_version, mod_loader))
}

pub fn load_instances(
    instances_dir: &PathBuf,
    groups: &HashMap<String, String>,
) -> Result<Vec<Instance>> {
    let mut instances = Vec::new();

    if !instances_dir.exists() {
        return Ok(instances);
    }

    for entry in fs::read_dir(instances_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Skip hidden directories and special folders
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "_MMC_TEMP" {
            continue;
        }

        // Check if it's a valid instance (has instance.cfg)
        if !path.join("instance.cfg").exists() {
            continue;
        }

        match Instance::load(path, groups) {
            Ok(instance) => instances.push(instance),
            Err(e) => eprintln!("Warning: Failed to load instance: {}", e),
        }
    }

    // Sort by last launch time (most recent first)
    instances.sort_by(|a, b| b.last_launch.cmp(&a.last_launch));

    Ok(instances)
}
