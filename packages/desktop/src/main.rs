// Prevent additional console window on Windows in release mode.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(async_closure)]
#![feature(iterator_try_collect)]

use std::{env, sync::Arc};

use clap::Parser;
use cli::MonitorType;
use config::{MonitorSelection, WidgetPlacement};
use providers::ProviderEmission;
use tauri::{
  async_runtime::block_on, AppHandle, Emitter, Manager, RunEvent,
};
use tauri_plugin_shell::ShellExt;
use tokio::{sync::mpsc, task};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;
use widget_factory::WidgetOpenOptions;

#[cfg(target_os = "windows")]
use crate::common::windows::WindowExtWindows;
use crate::{
  cli::{Cli, CliCommand, QueryArgs},
  config::Config,
  monitor_state::MonitorState,
  providers::ProviderManager,
  sys_tray::SysTray,
  widget_factory::WidgetFactory,
};

mod cli;
mod commands;
mod common;
mod config;
mod monitor_state;
mod providers;
mod sys_tray;
mod widget_factory;

/// Main entry point for the application.
///
/// Conditionally starts Zebar or runs a CLI command based on the given
/// subcommand.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Attach to parent console on Windows in release mode.
  #[cfg(all(windows, not(debug_assertions)))]
  {
    use windows::Win32::System::Console::{
      AttachConsole, ATTACH_PARENT_PROCESS,
    };
    let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
  }

  tauri::async_runtime::set(tokio::runtime::Handle::current());

  let app = tauri::Builder::default()
    .setup(|app| {
      task::block_in_place(|| {
        block_on(async move {
          let cli = Cli::parse();

          match cli.command() {
            CliCommand::Query(args) => output_query(app, args),
            _ => {
              let start_res = start_app(app, cli).await;

              // If unable to start Zebar, the error is fatal and a message
              // dialog is shown.
              if let Err(err) = &start_res {
                // TODO: Show error dialog.
                error!("{:?}", err);
              };

              start_res
            }
          }?;

          Ok(())
        })
      })
    })
    .invoke_handler(tauri::generate_handler![
      commands::widget_configs,
      commands::widget_states,
      commands::start_widget,
      commands::start_preset,
      commands::stop_preset,
      commands::update_widget_config,
      commands::listen_provider,
      commands::unlisten_provider,
      commands::call_provider_function,
      commands::set_always_on_top,
      commands::set_skip_taskbar
    ])
    .build(tauri::generate_context!())?;

  app.run(|app, event| {
    if let RunEvent::ExitRequested { code, api, .. } = &event {
      if code.is_none() {
        // Keep the message loop running even if all windows are closed.
        api.prevent_exit();
      } else {
        // Deallocate any appbars on Windows.
        #[cfg(target_os = "windows")]
        {
          for (_, window) in app.webview_windows() {
            let _ = window.as_ref().window().deallocate_app_bar();
          }
        }
      }
    }
  });

  Ok(())
}

/// Query state and print to the console.
fn output_query(app: &tauri::App, args: QueryArgs) -> anyhow::Result<()> {
  match args {
    QueryArgs::Monitors => {
      let monitors = MonitorState::new(&app.handle());
      cli::print_and_exit(monitors.output_str());
      Ok(())
    }
  }
}

/// Starts Zebar - either with a specific widget or all widgets.
async fn start_app(app: &mut tauri::App, cli: Cli) -> anyhow::Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::from_env("LOG_LEVEL")
        .add_directive(LevelFilter::INFO.into()),
    )
    .init();

  let config_dir_override = match cli.command() {
    CliCommand::Startup(args) => args.config_dir,
    _ => None,
  };

  // Initialize `Config` in Tauri state.
  let config = Arc::new(Config::new(app.handle(), config_dir_override)?);
  app.manage(config.clone());

  // Initialize `MonitorState` in Tauri state.
  let monitor_state = Arc::new(MonitorState::new(app.handle()));
  app.manage(monitor_state.clone());

  // Initialize `WidgetFactory` in Tauri state.
  let widget_factory = Arc::new(WidgetFactory::new(
    app.handle(),
    config.clone(),
    monitor_state.clone(),
  ));
  app.manage(widget_factory.clone());

  // If this is not the first instance of the app, this will emit within
  // the original instance and exit immediately. The CLI command is
  // guaranteed to be one of the open commands here.
  setup_single_instance(app, widget_factory.clone())?;

  // Prevent windows from showing up in the dock on MacOS.
  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Accessory);

  // Allow assets to be resolved from the config directory.
  app
    .asset_protocol_scope()
    .allow_directory(&config.config_dir, true)?;

  app.handle().plugin(tauri_plugin_shell::init())?;
  app.handle().plugin(tauri_plugin_http::init())?;
  app.handle().plugin(tauri_plugin_dialog::init())?;

  let shell = app.handle().shell();
  shell
    .command("echo")
    .args(["Hello from Rust!"])
    .output()
    .await
    .unwrap();

  // Initialize `ProviderManager` in Tauri state.
  let (manager, emit_rx) = ProviderManager::new(app.handle());
  app.manage(manager.clone());

  // Open widgets based on CLI command.
  open_widgets_by_cli_command(cli, widget_factory.clone()).await?;

  // Add application icon to system tray.
  let tray =
    SysTray::new(app.handle(), config.clone(), widget_factory.clone())
      .await?;

  listen_events(
    app.handle(),
    config,
    monitor_state,
    widget_factory,
    tray,
    manager,
    emit_rx,
  );

  Ok(())
}

fn listen_events(
  app_handle: &AppHandle,
  config: Arc<Config>,
  monitor_state: Arc<MonitorState>,
  widget_factory: Arc<WidgetFactory>,
  tray: SysTray,
  manager: Arc<ProviderManager>,
  mut emit_rx: mpsc::UnboundedReceiver<ProviderEmission>,
) {
  let app_handle = app_handle.clone();
  let mut widget_open_rx = widget_factory.open_tx.subscribe();
  let mut widget_close_rx = widget_factory.close_tx.subscribe();
  let mut settings_change_rx = config.settings_change_tx.subscribe();
  let mut monitors_change_rx = monitor_state.change_tx.subscribe();
  let mut widget_configs_change_rx =
    config.widget_configs_change_tx.subscribe();

  task::spawn(async move {
    loop {
      let res = tokio::select! {
        Ok(widget_state) = widget_open_rx.recv() => {
          info!("Widget opened.");
          tray.refresh().await;
          app_handle.emit("widget-opened", widget_state);
          Ok(())
        },
        Ok(widget_id) = widget_close_rx.recv() => {
          info!("Widget closed.");
          tray.refresh().await;
          app_handle.emit("widget-closed", widget_id);
          Ok(())
        },
        Ok(_) = settings_change_rx.recv() => {
          info!("Settings changed.");
          tray.refresh().await
        },
        Ok(_) = monitors_change_rx.recv() => {
          info!("Monitors changed.");
          widget_factory.relaunch_all().await
        },
        Ok(changed_configs) = widget_configs_change_rx.recv() => {
          info!("Widget configs changed.");
          widget_factory.relaunch_by_paths(&changed_configs.keys().cloned().collect()).await
        },
        Some(provider_emission) = emit_rx.recv() => {
          info!("Provider emission: {:?}", provider_emission);
          app_handle.emit("provider-emit", provider_emission.clone());
          manager.update_cache(provider_emission).await;
          Ok(())
        },
      };

      if let Err(err) = res {
        error!("{:?}", err);
      }
    }
  });
}

/// Setup single instance Tauri plugin.
fn setup_single_instance(
  app: &tauri::App,
  widget_factory: Arc<WidgetFactory>,
) -> anyhow::Result<()> {
  app.handle().plugin(tauri_plugin_single_instance::init(
    move |_, args, _| {
      let widget_factory = widget_factory.clone();

      task::spawn(async move {
        let res = match Cli::try_parse_from(args) {
          Ok(cli) => {
            // No-op if no subcommand is provided.
            if cli.command() != CliCommand::Empty {
              open_widgets_by_cli_command(cli, widget_factory).await
            } else {
              Ok(())
            }
          }
          _ => Err(anyhow::anyhow!("Failed to parse CLI arguments.")),
        };

        if let Err(err) = res {
          error!("{:?}", err);
        }
      });
    },
  ))?;

  Ok(())
}

/// Opens widgets based on CLI command.
async fn open_widgets_by_cli_command(
  cli: Cli,
  widget_factory: Arc<WidgetFactory>,
) -> anyhow::Result<()> {
  let res = match cli.command() {
    CliCommand::StartWidget(args) => {
      widget_factory
        .start_widget(
          &args.config_path,
          &WidgetOpenOptions::Standalone(WidgetPlacement {
            anchor: args.anchor,
            offset_x: args.offset_x,
            offset_y: args.offset_y,
            width: args.width,
            height: args.height,
            monitor_selection: match args.monitor_type {
              MonitorType::All => MonitorSelection::All,
              MonitorType::Primary => MonitorSelection::Primary,
              MonitorType::Secondary => MonitorSelection::Secondary,
            },
            dock_to_edge: Default::default(),
          }),
        )
        .await
    }
    CliCommand::StartPreset(args) => {
      widget_factory
        .start_widget(
          &args.config_path,
          &WidgetOpenOptions::Preset(args.preset_name),
        )
        .await
    }
    CliCommand::Startup(_) | CliCommand::Empty => {
      widget_factory.startup().await
    }
    _ => unreachable!(),
  };

  if let Err(err) = res {
    error!("Failed to open widgets: {:?}", err);
  }

  Ok(())
}
