use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};

use anyhow::{bail, Context};
use serde::Serialize;
use tauri::{
  AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition,
  PhysicalSize, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tokio::{
  sync::{broadcast, Mutex},
  task,
};
use tracing::{error, info};

use crate::{
  common::{PathExt, WindowExt},
  config::{Config, WindowAnchor, WindowConfig, WindowConfigEntry},
  monitor_state::MonitorState,
};

/// Manages the creation of Zebar windows.
pub struct WindowFactory {
  /// Handle to the Tauri application.
  app_handle: AppHandle,

  _close_rx: broadcast::Receiver<WindowState>,

  pub close_tx: broadcast::Sender<WindowState>,

  /// Reference to `Config`.
  config: Arc<Config>,

  _open_rx: broadcast::Receiver<WindowState>,

  pub open_tx: broadcast::Sender<WindowState>,

  /// Reference to `MonitorState`.
  ///
  /// Used for window positioning.
  monitor_state: Arc<MonitorState>,

  /// Running total of windows created.
  ///
  /// Used to generate unique window labels.
  window_count: Arc<AtomicU32>,

  /// Map of window labels to window states.
  window_states: Arc<Mutex<HashMap<String, WindowState>>>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
  /// Unique identifier for the window.
  ///
  /// Used as the window label.
  pub window_id: String,

  /// User-defined config for the window.
  pub config: WindowConfig,

  /// Absolute path to the window's config file.
  pub config_path: PathBuf,

  /// Absolute path to the window's HTML file.
  pub html_path: PathBuf,
}

impl WindowFactory {
  /// Creates a new `WindowFactory` instance.
  pub fn new(
    app_handle: &AppHandle,
    config: Arc<Config>,
    monitor_state: Arc<MonitorState>,
  ) -> Self {
    let (open_tx, _open_rx) = broadcast::channel(16);
    let (close_tx, _close_rx) = broadcast::channel(16);

    Self {
      app_handle: app_handle.clone(),
      _close_rx,
      close_tx,
      config,
      _open_rx,
      open_tx,
      monitor_state,
      window_count: Arc::new(AtomicU32::new(0)),
      window_states: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  /// Opens windows from a given config entry.
  pub async fn open(
    &self,
    config_entry: WindowConfigEntry,
  ) -> anyhow::Result<()> {
    let WindowConfigEntry {
      config,
      config_path,
      html_path,
    } = &config_entry;

    for (size, position) in self.window_placements(config).await {
      // Use running window count as a unique ID for the window.
      let new_count =
        self.window_count.fetch_add(1, Ordering::Relaxed) + 1;
      let window_id = new_count.to_string();

      if !html_path.exists() {
        bail!(
          "HTML file not found at {} for config {}.",
          html_path.display(),
          config_path.display()
        );
      }

      info!(
        "Creating window #{} from {}",
        new_count,
        config_path.display()
      );

      let webview_url = WebviewUrl::App(
        Self::to_asset_url(&html_path.to_unicode_string()).into(),
      );

      // Note that window label needs to be globally unique.
      let window = WebviewWindowBuilder::new(
        &self.app_handle,
        window_id.clone(),
        webview_url,
      )
      .title("Zebar")
      .inner_size(size.width, size.height)
      .position(position.x, position.y)
      .focused(config.launch_options.focused)
      .skip_taskbar(!config.launch_options.shown_in_taskbar)
      .visible_on_all_workspaces(true)
      .transparent(config.launch_options.transparent)
      .shadow(false)
      .decorations(false)
      .resizable(config.launch_options.resizable)
      .build()?;

      let state = WindowState {
        window_id: window_id.clone(),
        config: config.clone(),
        config_path: config_path.clone(),
        html_path: html_path.clone(),
      };

      _ = window.eval(&format!(
        "window.__ZEBAR_INITIAL_STATE={}",
        serde_json::to_string(&state)?
      ));

      // On Windows, Tauri's `skip_taskbar` option isn't 100% reliable, so
      // we also set the window as a tool window.
      #[cfg(target_os = "windows")]
      let _ = window
        .as_ref()
        .window()
        .set_tool_window(!config.launch_options.shown_in_taskbar);

      // On Windows, there's an issue where the window size is constrained
      // when initially created. To work around this, apply the size and
      // position settings again after launch.
      #[cfg(target_os = "windows")]
      {
        let _ = window.set_size(size);
        let _ = window.set_position(position);
      }

      let mut window_states = self.window_states.lock().await;
      window_states.insert(state.window_id.clone(), state.clone());

      self.register_window_events(&window, window_id);
      self.open_tx.send(state)?;
    }

    Ok(())
  }

  /// Converts a file path to a Tauri asset URL.
  ///
  /// Returns a string that can be used as a webview URL.
  pub fn to_asset_url(file_path: &str) -> String {
    if cfg!(target_os = "windows") {
      format!("http://asset.localhost/{}", file_path)
    } else {
      format!("asset://localhost/{}", file_path)
    }
  }

  /// Registers window events for a given window.
  fn register_window_events(
    &self,
    window: &tauri::WebviewWindow,
    window_id: String,
  ) {
    let window_states = self.window_states.clone();
    let close_tx = self.close_tx.clone();

    window.on_window_event(move |event| {
      if let WindowEvent::Destroyed = event {
        let window_states = window_states.clone();
        let close_tx = close_tx.clone();
        let window_id = window_id.clone();

        task::spawn(async move {
          let mut window_states = window_states.lock().await;

          // Remove the window state and broadcast the close event.
          if let Some(state) = window_states.remove(&window_id) {
            if let Err(err) = close_tx.send(state) {
              error!("Failed to send window close event: {:?}", err);
            }
          }
        });
      }
    });
  }

  /// Returns coordinates for window placement based on the given config.
  async fn window_placements(
    &self,
    config: &WindowConfig,
  ) -> Vec<(LogicalSize<f64>, LogicalPosition<f64>)> {
    let mut placements = vec![];

    for placement in config.launch_options.placements.iter() {
      let monitors = self
        .monitor_state
        .monitors_by_selection(&placement.monitor_selection)
        .await;

      for monitor in monitors {
        let (anchor_x, anchor_y) = match placement.anchor {
          WindowAnchor::TopLeft => (monitor.x, monitor.y),
          WindowAnchor::TopCenter => {
            (monitor.x + (monitor.width as i32 / 2), monitor.y)
          }
          WindowAnchor::TopRight => {
            (monitor.x + monitor.width as i32, monitor.y)
          }
          WindowAnchor::CenterLeft => {
            (monitor.x, monitor.y + (monitor.height as i32 / 2))
          }
          WindowAnchor::Center => (
            monitor.x + (monitor.width as i32 / 2),
            monitor.y + (monitor.height as i32 / 2),
          ),
          WindowAnchor::CenterRight => (
            monitor.x + monitor.width as i32,
            monitor.y + (monitor.height as i32 / 2),
          ),
          WindowAnchor::BottomLeft => {
            (monitor.x, monitor.y + monitor.height as i32)
          }
          WindowAnchor::BottomCenter => (
            monitor.x + (monitor.width as i32 / 2),
            monitor.y + monitor.height as i32,
          ),
          WindowAnchor::BottomRight => (
            monitor.x + monitor.width as i32,
            monitor.y + monitor.height as i32,
          ),
        };

        let size = LogicalSize::from_physical(
          PhysicalSize::new(
            placement.width.to_px(monitor.width as i32),
            placement.height.to_px(monitor.height as i32),
          ),
          monitor.scale_factor,
        );

        let position = LogicalPosition::from_physical(
          PhysicalPosition::new(
            anchor_x + placement.offset_x.to_px(monitor.width as i32),
            anchor_y + placement.offset_y.to_px(monitor.height as i32),
          ),
          monitor.scale_factor,
        );

        placements.push((size, position));
      }
    }

    placements
  }

  /// Closes a single window by a given window ID.
  pub fn close_by_id(&self, window_id: &str) -> anyhow::Result<()> {
    let window = self
      .app_handle
      .get_webview_window(window_id)
      .context("No window found with the given ID.")?;

    window.close()?;

    Ok(())
  }

  /// Closes all windows with the given config path.
  pub async fn close_by_path(
    &self,
    config_path: &PathBuf,
  ) -> anyhow::Result<()> {
    let window_states = self.states_by_config_path().await;

    let found_window_states = window_states
      .get(config_path)
      .context("No windows found with the given config path.")?;

    for window_state in found_window_states {
      self.close_by_id(&window_state.window_id)?;
    }

    Ok(())
  }

  /// Relaunches all currently open windows.
  pub async fn relaunch_all(&self) -> anyhow::Result<()> {
    let window_states = self.states_by_config_path().await;

    for (config_path, _) in window_states {
      let _ = self.close_by_path(&config_path).await;

      let window_config = self
        .config
        .window_config_by_path(&config_path)
        .await?
        .context("Window config not found.")?;

      self.open(window_config).await?;
    }

    Ok(())
  }

  /// Returns window state by a given window ID.
  pub async fn state_by_id(&self, window_id: &str) -> Option<WindowState> {
    self.window_states.lock().await.get(window_id).cloned()
  }

  /// Returns window states grouped by their config paths.
  pub async fn states_by_config_path(
    &self,
  ) -> HashMap<PathBuf, Vec<WindowState>> {
    self.window_states.lock().await.values().fold(
      HashMap::new(),
      |mut acc, state| {
        acc
          .entry(state.config_path.clone())
          .or_insert_with(Vec::new)
          .push(state.clone());

        acc
      },
    )
  }
}
