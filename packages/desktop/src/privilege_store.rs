use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Context;
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tokio::{fs, sync::Mutex};

use crate::config::WidgetPrivileges;

pub struct PrivilegeStore {
  /// Directory containing the privilege files.
  privileges_dir: PathBuf,

  /// Cache of widget privileges.
  cache: Arc<Mutex<HashMap<PathBuf, WidgetPrivileges>>>,
}

impl PrivilegeStore {
  /// Creates a new `PrivilegeStore`.
  pub fn new(app_handle: &AppHandle) -> anyhow::Result<Self> {
    let privileges_dir = app_handle
      .path()
      .resolve("privileges", BaseDirectory::AppData)
      .context("Unable to get data directory.")?;

    Ok(Self {
      privileges_dir,
      cache: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  pub async fn validate(
    &self,
    widget_path: &PathBuf,
    privileges: &WidgetPrivileges,
  ) -> anyhow::Result<bool> {
    let found_privileges = self.get(widget_path).await?;
    Ok(found_privileges == privileges)
  }

  /// Gets the privileges for a widget.
  async fn get(&self, widget_path: &PathBuf) -> Option<WidgetPrivileges> {
    // Try to get the privileges from the cache.
    {
      let cache = self.cache.lock().await;

      if let Some(privileges) = cache.get(widget_path) {
        return Some(privileges.clone());
      }
    }

    // Read the privileges from the file.
    let privileges =
      fs::read_to_string(self.privileges_dir.join(widget_path))
        .await
        .context("Unable to read privileges file.")?;

    None
  }

  /// Clears the cache.
  pub async fn clear_cache(&self) {
    let mut cache = self.cache.lock().await;
    cache.clear();
  }
}
