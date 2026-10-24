use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Persisted CLI state, stored under the user configuration directory.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: Option<String>,
    pub token: Option<String>,
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let base =
            dirs::config_dir().context("could not determine the user configuration directory")?;
        Ok(base.join("openlist-cli").join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        write_private(&path, contents.as_bytes())
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn set_credentials(&mut self, server: impl Into<String>, token: impl Into<String>) {
        self.server = Some(server.into());
        self.token = Some(token.into());
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    fs::write(path, bytes)
}
