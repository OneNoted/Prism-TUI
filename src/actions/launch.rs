use crate::error::{PrismError, Result};
use std::process::{Command, Stdio};

pub fn launch_instance(
    instance_id: &str,
    account: Option<&str>,
    server: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("prismlauncher");

    // Detach process output from TUI
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd.arg("--launch").arg(instance_id);

    if let Some(profile) = account {
        cmd.arg("--profile").arg(profile);
    }

    if let Some(server_addr) = server {
        cmd.arg("--server").arg(server_addr);
    }

    cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PrismError::LaunchFailed("prismlauncher not found in PATH".into())
        } else {
            PrismError::LaunchFailed(e.to_string())
        }
    })?;

    Ok(())
}
