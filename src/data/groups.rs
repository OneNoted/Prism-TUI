use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct GroupsFile {
    groups: HashMap<String, GroupEntry>,
}

#[derive(Deserialize)]
struct GroupEntry {
    hidden: bool,
    instances: Vec<String>,
}

/// Load instance groups and return a map of instance_id -> group_name
pub fn load_groups(instances_dir: &Path) -> Result<HashMap<String, String>> {
    let groups_path = instances_dir.join("instgroups.json");
    let mut instance_to_group = HashMap::new();

    if !groups_path.exists() {
        return Ok(instance_to_group);
    }

    let content = fs::read_to_string(&groups_path)?;
    let groups_file: GroupsFile = serde_json::from_str(&content)?;

    for (group_name, group_entry) in groups_file.groups {
        if group_entry.hidden {
            continue;
        }
        for instance_id in group_entry.instances {
            instance_to_group.insert(instance_id, group_name.clone());
        }
    }

    Ok(instance_to_group)
}
