use crate::error::Result;
use hematite_nbt::{Blob, Value};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
}

pub fn load_servers(servers_dat_path: &PathBuf) -> Result<Vec<Server>> {
    if !servers_dat_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(servers_dat_path)?;
    let mut reader = BufReader::new(file);
    let blob = Blob::from_reader(&mut reader)?;

    let mut servers = Vec::new();

    if let Some(Value::List(server_list)) = blob.get("servers") {
        for server_value in server_list {
            if let Value::Compound(server_map) = server_value {
                let name = match server_map.get("name") {
                    Some(Value::String(s)) => s.clone(),
                    _ => "Unknown".to_string(),
                };
                let ip = match server_map.get("ip") {
                    Some(Value::String(s)) => s.clone(),
                    _ => continue,
                };
                servers.push(Server { name, ip });
            }
        }
    }

    Ok(servers)
}

pub fn save_servers(servers_dat_path: &PathBuf, servers: &[Server]) -> Result<()> {
    let mut blob = Blob::new();

    let server_list: Vec<Value> = servers
        .iter()
        .map(|server| {
            let mut map = std::collections::HashMap::new();
            map.insert("name".to_string(), Value::String(server.name.clone()));
            map.insert("ip".to_string(), Value::String(server.ip.clone()));
            Value::Compound(map)
        })
        .collect();

    blob.insert("servers", Value::List(server_list))?;

    // Ensure parent directory exists
    if let Some(parent) = servers_dat_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(servers_dat_path)?;
    let mut writer = BufWriter::new(file);
    blob.to_writer(&mut writer)?;

    Ok(())
}
