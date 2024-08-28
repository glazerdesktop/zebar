use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::{bail, Context};
use tauri::{
  image::Image,
  menu::{CheckMenuItem, Menu, MenuBuilder, Submenu, SubmenuBuilder},
  tray::{TrayIcon, TrayIconBuilder},
  AppHandle, Wry,
};
use tracing::{error, info};

use crate::{
  config::{Config, WindowConfig, WindowConfigEntry},
  window_factory::WindowFactory,
};

#[derive(Debug, Clone)]
enum MenuEvent {
  ShowConfigFolder,
  Exit,
  EnableWindowConfig(usize),
  StartupWindowConfig(usize),
}

impl ToString for MenuEvent {
  fn to_string(&self) -> String {
    match self {
      MenuEvent::ShowConfigFolder => "show_config_folder".to_string(),
      MenuEvent::Exit => "exit".to_string(),
      MenuEvent::EnableWindowConfig(id) => format!("enable_{}", id),
      MenuEvent::StartupWindowConfig(id) => format!("startup_{}", id),
    }
  }
}

impl FromStr for MenuEvent {
  type Err = anyhow::Error;

  fn from_str(event: &str) -> Result<Self, Self::Err> {
    if event == "show_config_folder" {
      Ok(MenuEvent::ShowConfigFolder)
    } else if event == "exit" {
      Ok(MenuEvent::Exit)
    } else if let Some(id) = event.strip_prefix("enable_") {
      Ok(MenuEvent::EnableWindowConfig(id.parse()?))
    } else if let Some(id) = event.strip_prefix("startup_") {
      Ok(MenuEvent::StartupWindowConfig(id.parse()?))
    } else {
      bail!("Invalid menu event: {}", event)
    }
  }
}

/// System tray icon for Zebar.
pub struct SysTray {}

impl SysTray {
  /// Creates a new system tray icon for Zebar.
  pub async fn new(
    app_handle: &AppHandle,
    config: Arc<Config>,
    window_factory: Arc<WindowFactory>,
  ) -> anyhow::Result<TrayIcon> {
    let tooltip = format!("Zebar v{}", env!("VERSION_NUMBER"));

    let tray_menu =
      Self::tray_menu(app_handle, config.clone(), window_factory.clone())
        .await?;

    let tray_icon = TrayIconBuilder::with_id("tray")
      .icon(Self::icon_image(app_handle)?)
      .menu(&tray_menu)
      .tooltip(tooltip)
      .on_menu_event(move |app, event| {
        let event = MenuEvent::from_str(event.id.as_ref());

        let event_res = event.map(|event| {
          Self::handle_menu_event(event, app, config.clone())
        });

        if let Err(err) = event_res {
          error!("{:?}", err);
        }
      })
      .build(app_handle)?;

    Ok(tray_icon)
  }

  /// Returns the image to use for the system tray icon.
  fn icon_image(app_handle: &AppHandle) -> anyhow::Result<Image> {
    app_handle
      .default_window_icon()
      .cloned()
      .context("No icon defined in Tauri config.")
  }

  /// Creates and returns the main system tray menu.
  async fn tray_menu(
    app_handle: &AppHandle,
    config: Arc<Config>,
    window_factory: Arc<WindowFactory>,
  ) -> anyhow::Result<Menu<Wry>> {
    let mut tray_menu = MenuBuilder::new(app_handle)
      .item(&Self::configs_menu(app_handle, config.clone())?)
      .text(MenuEvent::ShowConfigFolder, "Show config folder")
      .separator();

    let window_states = window_factory.states_by_config_path().await;

    // If there are active windows, add them to the menu.
    if !window_states.is_empty() {
      for (id, state) in window_states {
        let config_path =
          Self::format_config_path(&id, &config.config_dir);
        let label = format!("({}) {}", state.len(), config_path);

        tray_menu = tray_menu.item(&Self::config_menu(
          app_handle,
          &state.get(0).unwrap().config,
          0,
          &label,
        )?);
      }

      tray_menu = tray_menu.separator();
    }

    tray_menu = tray_menu.text(MenuEvent::Exit, "Exit");

    Ok(tray_menu.build()?)
  }

  /// Callback for system tray menu events.
  fn handle_menu_event(
    event: MenuEvent,
    app_handle: &AppHandle,
    config: Arc<Config>,
  ) -> anyhow::Result<()> {
    match event {
      MenuEvent::ShowConfigFolder => {
        info!("Opening config folder from system tray.");

        config
          .open_config_dir()
          .context("Failed to open config folder.")?;
      }
      MenuEvent::Exit => {
        info!("Exiting through system tray.");

        app_handle.exit(0)
      }
      MenuEvent::EnableWindowConfig(id) => {
        info!("Window config at index {} enabled.", id);
      }
      MenuEvent::StartupWindowConfig(id) => {
        info!("Window config at index {} set to launch on startup.", id);
      }
    };

    Ok(())
  }

  /// Creates and returns a submenu for the window configs.
  fn configs_menu(
    app_handle: &AppHandle,
    config: Arc<Config>,
  ) -> anyhow::Result<Submenu<Wry>> {
    let configs_menu = SubmenuBuilder::new(app_handle, "Window configs");

    // Add each window config to the menu.
    let configs_menu = config
      .window_configs
      .iter()
      .enumerate()
      .try_fold(configs_menu, |menu, (index, window_config)| {
        let label = Self::format_config_path(
          &window_config.config_path,
          &config.config_dir,
        );

        let item = Self::config_menu(
          app_handle,
          &window_config.config,
          index,
          &label,
        )?;

        anyhow::Ok(menu.item(&item))
      })?
      .build()?;

    Ok(configs_menu)
  }

  /// Creates and returns a submenu for the given window config.
  fn config_menu(
    app_handle: &AppHandle,
    config: &WindowConfig,
    index: usize,
    label: &str,
  ) -> anyhow::Result<Submenu<Wry>> {
    // TODO: Get whether it's enabled.
    let enabled_item = CheckMenuItem::with_id(
      app_handle,
      MenuEvent::EnableWindowConfig(index),
      "Enabled",
      true,
      true,
      None::<&str>,
    )?;

    // TODO: Get whether it's launched on startup.
    let startup_item = CheckMenuItem::with_id(
      app_handle,
      MenuEvent::StartupWindowConfig(index),
      "Launch on startup",
      true,
      true,
      None::<&str>,
    )?;

    let config_menu = SubmenuBuilder::new(app_handle, label)
      .item(&enabled_item)
      .item(&startup_item)
      .build()?;

    Ok(config_menu)
  }

  /// Formats the config path for display in the system tray.
  fn format_config_path(
    config_path: &str,
    config_dir: &PathBuf,
  ) -> String {
    let config_pathbuf = PathBuf::from(config_path);

    config_pathbuf
      .strip_prefix(config_dir)
      .unwrap_or(&config_pathbuf)
      .with_extension("")
      .to_string_lossy()
      .to_string()
  }
}
