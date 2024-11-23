use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Context;
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tokio::sync::Mutex;

use crate::{common::read_and_parse_json, config::WidgetPrivileges};

pub struct PrivilegeStore {
  /// The app handle.
  app_handle: AppHandle,

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
      app_handle: app_handle.clone(),
      privileges_dir,
      cache: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  /// Validates the privileges for a widget.
  ///
  /// If the privileges are not found in the cache, the user will be
  /// prompted to grant them.
  pub async fn validate_or_prompt(
    &self,
    widget_path: &PathBuf,
    privileges: &WidgetPrivileges,
  ) -> anyhow::Result<bool> {
    let matches = self
      .fetch_privileges(widget_path)
      .await
      .ok()
      .map(|found_privileges| {
        found_privileges.as_ref() == Some(privileges)
      })
      .unwrap_or(false);

    match matches {
      true => Ok(true),
      false => self.prompt_for_privileges(widget_path, privileges),
    }
  }

  /// Fetches the privileges for a widget.
  async fn fetch_privileges(
    &self,
    widget_path: &PathBuf,
  ) -> anyhow::Result<Option<WidgetPrivileges>> {
    // Try to get the privileges from the cache.
    {
      let cache = self.cache.lock().await;

      if let Some(privileges) = cache.get(widget_path) {
        return Ok(Some(privileges.clone()));
      }
    }

    // Read the privileges from the file.
    let privileges = read_and_parse_json::<WidgetPrivileges>(
      &self.privileges_dir.join(widget_path),
    )
    .context("Unable to read privileges file.")?;

    Ok(Some(privileges))
  }

  /// Clears the cache.
  pub async fn clear_cache(&self) {
    let mut cache = self.cache.lock().await;
    cache.clear();
  }

  /// Prompts the user to grant privileges for a widget.
  fn prompt_for_privileges(
    &self,
    widget_path: &PathBuf,
    privileges: &WidgetPrivileges,
  ) -> anyhow::Result<bool> {
    let answer = self
      .app_handle
      .dialog()
      .message(format!(
        "      Permission Request      \n\n\
        This widget needs to call external commands:\n\n\
        Location:\n    {}\n\n\
        Details:\n\
        ○  Command:  {}\n\
        ○  Pattern:  {}\n\n\
        Is this okay?",
        widget_path.display(),
        privileges.shell[0].program,
        privileges.shell[0].args_regex
      ))
      .title("Privileges")
      .buttons(MessageDialogButtons::OkCancelCustom(
        "Reject".to_string(),
        "Accept".to_string(),
      ))
      .blocking_show();

    Ok(answer)
  }
}
