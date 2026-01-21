use crate::error::{PrismError, Result};
use std::env;
use std::path::Path;
use std::process::Command;

pub fn open_folder(path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";

    #[cfg(target_os = "macos")]
    let opener = "open";

    #[cfg(target_os = "windows")]
    let opener = "explorer";

    Command::new(opener)
        .arg(path)
        .spawn()
        .map_err(|e| PrismError::Other(format!("Failed to open folder: {}", e)))?;

    Ok(())
}

pub fn open_in_editor(path: &Path) -> Result<()> {
    // Try $EDITOR first, then fall back to xdg-open/platform opener
    let editor = env::var("EDITOR").ok();

    if let Some(editor) = editor {
        Command::new(&editor)
            .arg(path)
            .spawn()
            .map_err(|e| PrismError::Other(format!("Failed to open editor '{}': {}", editor, e)))?;
    } else {
        // Fall back to platform opener
        #[cfg(target_os = "linux")]
        let opener = "xdg-open";

        #[cfg(target_os = "macos")]
        let opener = "open";

        #[cfg(target_os = "windows")]
        let opener = "notepad";

        Command::new(opener)
            .arg(path)
            .spawn()
            .map_err(|e| PrismError::Other(format!("Failed to open file: {}", e)))?;
    }

    Ok(())
}
