use crate::error::Result;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub name: String,
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
    pub size: u64,
}

impl LogEntry {
    pub fn formatted_size(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        }
    }
}

pub fn load_log_entries(dir: &Path) -> Result<Vec<LogEntry>> {
    let mut entries = Vec::new();

    if !dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Only include .log and .log.gz files
        if !name.ends_with(".log") && !name.ends_with(".log.gz") {
            continue;
        }

        let metadata = entry.metadata()?;
        let modified = metadata.modified().ok();
        let size = metadata.len();

        entries.push(LogEntry {
            name,
            path,
            modified,
            size,
        });
    }

    // Sort by modified time (most recent first), with latest.log always first
    entries.sort_by(|a, b| {
        // latest.log always comes first
        if a.name == "latest.log" {
            return std::cmp::Ordering::Less;
        }
        if b.name == "latest.log" {
            return std::cmp::Ordering::Greater;
        }

        // Then sort by modified time (most recent first)
        match (&b.modified, &a.modified) {
            (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });

    Ok(entries)
}

/// Maximum decompressed log file size (10 MB)
const MAX_LOG_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of lines to read from a log file
const MAX_LOG_LINES: usize = 100_000;

pub fn load_log_content(path: &Path) -> Result<Vec<String>> {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    let lines = if name.ends_with(".gz") {
        // Decompress gzip file with size limit
        let file = File::open(path)?;
        let decoder = GzDecoder::new(file);
        let reader = BufReader::new(decoder.take(MAX_LOG_SIZE as u64));
        reader
            .lines()
            .take(MAX_LOG_LINES)
            .collect::<std::io::Result<Vec<_>>>()?
    } else {
        // Read plain text file with line limit
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        reader
            .lines()
            .take(MAX_LOG_LINES)
            .collect::<std::io::Result<Vec<_>>>()?
    };

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_formatted_size_bytes() {
        let entry = LogEntry {
            name: "test.log".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            modified: None,
            size: 512,
        };
        assert_eq!(entry.formatted_size(), "512 B");
    }

    #[test]
    fn test_log_entry_formatted_size_kilobytes() {
        let entry = LogEntry {
            name: "test.log".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            modified: None,
            size: 2048,
        };
        assert_eq!(entry.formatted_size(), "2.0 KB");
    }

    #[test]
    fn test_log_entry_formatted_size_megabytes() {
        let entry = LogEntry {
            name: "test.log".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            modified: None,
            size: 5 * 1024 * 1024,
        };
        assert_eq!(entry.formatted_size(), "5.0 MB");
    }

    #[test]
    fn test_load_log_entries_empty_dir() {
        let result = load_log_entries(Path::new("/nonexistent/path"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
