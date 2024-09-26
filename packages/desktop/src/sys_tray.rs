use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::{bail, Context};
use tauri::{
  image::Image,
  menu::{CheckMenuItem, Menu, MenuBuilder, Submenu, SubmenuBuilder},
  tray::{TrayIcon, TrayIconBuilder},
  AppHandle, Wry,
};
use tokio::task;
use tracing::{error, info};

use crate::{
  common::PathExt,
  config::Config,
  widget_factory::{WidgetFactory, WidgetState},
};

#[derive(Debug, Clone)]
enum MenuEvent {
  ShowConfigFolder,
  ReloadConfigs,
  Exit,
  ToggleWidgetConfig { enable: bool, path: PathBuf },
  ToggleStartupWidgetConfig { enable: bool, path: PathBuf },
}

impl ToString for MenuEvent {
  fn to_string(&self) -> String {
    match self {
      MenuEvent::ShowConfigFolder => "show_config_folder".to_string(),
      MenuEvent::ReloadConfigs => "reload_configs".to_string(),
      MenuEvent::Exit => "exit".to_string(),
      MenuEvent::ToggleWidgetConfig { enable, path } => {
        format!(
          "toggle_widget_config_{}_{}",
          enable,
          path.to_unicode_string()
        )
      }
      MenuEvent::ToggleStartupWidgetConfig { enable, path } => {
        format!(
          "toggle_startup_widget_config_{}_{}",
          enable,
          path.to_unicode_string()
        )
      }
    }
  }
}

impl FromStr for MenuEvent {
  type Err = anyhow::Error;

  fn from_str(event: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = event.split('_').collect();

    match parts.as_slice() {
      ["show", "config", "folder"] => Ok(Self::ShowConfigFolder),
      ["reload", "configs"] => Ok(Self::ReloadConfigs),
      ["exit"] => Ok(Self::Exit),
      ["toggle", "widget", "config", enable @ ("true" | "false"), path @ ..] => {
        Ok(Self::ToggleWidgetConfig {
          enable: *enable == "true",
          path: PathBuf::from(path.join("_")),
        })
      }
      ["toggle", "startup", "widget", "config", enable @ ("true" | "false"), path @ ..] => {
        Ok(Self::ToggleStartupWidgetConfig {
          enable: *enable == "true",
          path: PathBuf::from(path.join("_")),
        })
      }
      _ => bail!("Invalid menu event: {}", event),
    }
  }
}

/// System tray icon for Zebar.
pub struct SysTray {
  app_handle: AppHandle,
  config: Arc<Config>,
  widget_factory: Arc<WidgetFactory>,
  tray_icon: Option<TrayIcon>,
}

impl SysTray {
  /// Creates a new system tray icon for Zebar.
  pub async fn new(
    app_handle: &AppHandle,
    config: Arc<Config>,
    widget_factory: Arc<WidgetFactory>,
  ) -> anyhow::Result<Arc<SysTray>> {
    let mut sys_tray = Self {
      app_handle: app_handle.clone(),
      config,
      widget_factory,
      tray_icon: None,
    };

    sys_tray.tray_icon = Some(sys_tray.create_tray_icon().await?);

    Ok(Arc::new(sys_tray))
  }

  async fn create_tray_icon(&self) -> anyhow::Result<TrayIcon> {
    let config = self.config.clone();
    let widget_factory = self.widget_factory.clone();
    let tooltip = format!("Zebar v{}", env!("VERSION_NUMBER"));

    let tray_icon = TrayIconBuilder::with_id("tray")
      .icon(self.icon_image()?)
      .menu(&self.create_tray_menu().await?)
      .tooltip(tooltip)
      .on_menu_event(move |app, event| {
        let app_handle = app.clone();
        let config = config.clone();
        let widget_factory = widget_factory.clone();

        task::spawn(async move {
          let event = MenuEvent::from_str(event.id.as_ref());

          if let Ok(event) = event {
            info!("Received tray menu event: {}", event.to_string());

            let res = Self::handle_menu_event(
              event,
              app_handle,
              config,
              widget_factory,
            )
            .await;

            if let Err(err) = res {
              error!("{:?}", err);
            }
          }
        });
      })
      .build(&self.app_handle)?;

    Ok(tray_icon)
  }

  pub async fn refresh(&self) -> anyhow::Result<()> {
    info!("Updating system tray menu.");

    if let Some(tray_icon) = self.tray_icon.as_ref() {
      let tray_menu = self.create_tray_menu().await?;
      tray_icon.set_menu(Some(tray_menu))?;
    }

    Ok(())
  }

  /// Returns the image to use for the system tray icon.
  fn icon_image(&self) -> anyhow::Result<Image> {
    self
      .app_handle
      .default_window_icon()
      .cloned()
      .context("No icon defined in Tauri config.")
  }

  /// Creates and returns the main system tray menu.
  async fn create_tray_menu(&self) -> anyhow::Result<Menu<Wry>> {
    let widget_states = self.widget_factory.states_by_config_path().await;

    let startup_config_paths = self
      .config
      .startup_widget_configs()
      .await?
      .into_iter()
      .map(|entry| entry.config_path)
      .collect();

    let configs_menu = self
      .create_configs_menu(&widget_states, &startup_config_paths)
      .await?;

    let mut tray_menu = MenuBuilder::new(&self.app_handle)
      .item(&configs_menu)
      .text(MenuEvent::ShowConfigFolder, "Show config folder")
      .text(MenuEvent::ReloadConfigs, "Reload configs")
      .separator();

    // Add submenus for currently active widget.
    if !widget_states.is_empty() {
      for (config_path, states) in widget_states {
        let label = format!(
          "({}) {}",
          states.len(),
          Self::format_config_path(&self.config, &config_path)
        );

        tray_menu = tray_menu.item(&self.create_config_menu(
          &config_path,
          &label,
          !states.is_empty(),
          startup_config_paths.contains(&config_path),
        )?);
      }

      tray_menu = tray_menu.separator();
    }

    tray_menu = tray_menu.text(MenuEvent::Exit, "Exit");

    Ok(tray_menu.build()?)
  }

  /// Callback for system tray menu events.
  async fn handle_menu_event(
    event: MenuEvent,
    app_handle: AppHandle,
    config: Arc<Config>,
    widget_factory: Arc<WidgetFactory>,
  ) -> anyhow::Result<()> {
    match event {
      MenuEvent::ShowConfigFolder => {
        info!("Opening config folder from system tray.");

        config
          .open_config_dir()
          .context("Failed to open config folder.")
      }
      MenuEvent::ReloadConfigs => {
        info!("Opening config folder from system tray.");

        config.reload().await
      }
      MenuEvent::Exit => {
        info!("Exiting through system tray.");

        app_handle.exit(0);

        Ok(())
      }
      MenuEvent::ToggleWidgetConfig { enable, path } => {
        info!(
          "Widget config '{}' to be enabled: {}",
          path.display(),
          enable
        );

        match enable {
          true => {
            let widget_config = config
              .widget_config_by_path(&path)
              .await?
              .context("Widget config not found.")?;

            widget_factory.open(widget_config).await
          }
          false => widget_factory.stop_by_path(&path).await,
        }
      }
      MenuEvent::ToggleStartupWidgetConfig { enable, path } => {
        info!(
          "Widget config '{}' to be launched on startup: {}",
          path.display(),
          enable
        );

        match enable {
          true => config.add_startup_config(&path).await,
          false => config.remove_startup_config(&path).await,
        }
      }
    }
  }

  /// Creates and returns a submenu for the widget configs.
  async fn create_configs_menu(
    &self,
    widget_states: &HashMap<PathBuf, Vec<WidgetState>>,
    startup_config_paths: &Vec<PathBuf>,
  ) -> anyhow::Result<Submenu<Wry>> {
    let mut configs_menu =
      SubmenuBuilder::new(&self.app_handle, "Widget configs");

    // Add each widget config to the menu.
    for widget_config in &self.config.widget_configs().await {
      let label =
        Self::format_config_path(&self.config, &widget_config.config_path);

      let menu_item = self.create_config_menu(
        &widget_config.config_path,
        &label,
        widget_states.contains_key(&widget_config.config_path),
        startup_config_paths.contains(&widget_config.config_path),
      )?;

      configs_menu = configs_menu.item(&menu_item);
    }

    Ok(configs_menu.build()?)
  }

  /// Creates and returns a submenu for the given widget config.
  fn create_config_menu(
    &self,
    config_path: &PathBuf,
    label: &str,
    is_enabled: bool,
    is_launched_on_startup: bool,
  ) -> anyhow::Result<Submenu<Wry>> {
    let enabled_item = CheckMenuItem::with_id(
      &self.app_handle,
      MenuEvent::ToggleWidgetConfig {
        enable: !is_enabled,
        path: config_path.clone(),
      },
      "Enabled",
      true,
      is_enabled,
      None::<&str>,
    )?;

    let startup_item = CheckMenuItem::with_id(
      &self.app_handle,
      MenuEvent::ToggleStartupWidgetConfig {
        enable: !is_launched_on_startup,
        path: config_path.clone(),
      },
      "Launch on startup",
      true,
      is_launched_on_startup,
      None::<&str>,
    )?;

    let config_menu = SubmenuBuilder::new(&self.app_handle, label)
      .item(&enabled_item)
      .item(&startup_item);

    Ok(config_menu.build()?)
  }

  /// Formats the config path for display in the system tray.
  fn format_config_path(
    config: &Arc<Config>,
    config_path: &PathBuf,
  ) -> String {
    let path = config
      .strip_config_dir(config_path)
      .unwrap_or(config_path.clone())
      .to_unicode_string();

    path.strip_suffix(".zebar.json").unwrap_or(&path).into()
  }
}
