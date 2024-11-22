use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Context};
use serde::{ser::SerializeStruct, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::{
  sync::{mpsc, oneshot, Mutex},
  task,
};
use tracing::info;

#[cfg(windows)]
use super::{
  audio::AudioProvider, focused_window::FocusedWindowProvider,
  keyboard::KeyboardProvider, komorebi::KomorebiProvider,
  media::MediaProvider,
};
use super::{
  battery::BatteryProvider, cpu::CpuProvider, disk::DiskProvider,
  host::HostProvider, ip::IpProvider, memory::MemoryProvider,
  network::NetworkProvider, weather::WeatherProvider, Provider,
  ProviderConfig, ProviderFunction, ProviderFunctionResponse,
  ProviderFunctionResult, ProviderOutput, RuntimeType,
};

/// Common fields for a provider.
pub struct CommonProviderState {
  /// Wrapper around the sender channel of provider emissions.
  pub emitter: ProviderEmitter,

  /// Wrapper around the receiver channel for incoming inputs to the
  /// provider.
  pub input: ProviderInput,

  /// Shared `sysinfo` instance.
  pub sysinfo: Arc<Mutex<sysinfo::System>>,
}

/// Handle for receiving provider inputs.
pub struct ProviderInput {
  /// Async receiver channel for incoming inputs to the provider.
  pub async_rx: mpsc::Receiver<ProviderInputMsg>,

  /// Sync receiver channel for incoming inputs to the provider.
  pub sync_rx: crossbeam::channel::Receiver<ProviderInputMsg>,
}

pub enum ProviderInputMsg {
  Function(ProviderFunction, oneshot::Sender<ProviderFunctionResult>),
  Stop,
}

/// Handle for sending provider emissions.
#[derive(Clone, Debug)]
pub struct ProviderEmitter {
  /// Sender channel for outgoing provider emissions.
  emit_tx: mpsc::UnboundedSender<ProviderEmission>,

  /// Hash of the provider's config.
  config_hash: String,
}

impl ProviderEmitter {
  /// Emits an output from a provider.
  pub fn emit_output<T>(&self, output: anyhow::Result<T>)
  where
    T: Into<ProviderOutput>,
  {
    let send_res = self.emit_tx.send(ProviderEmission {
      config_hash: self.config_hash.clone(),
      result: output.map(Into::into).map_err(|err| err.to_string()),
    });

    if let Err(err) = send_res {
      tracing::error!("Error sending provider result: {:?}", err);
    }
  }
}

/// Emission from a provider.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEmission {
  /// Hash of the provider's config.
  pub config_hash: String,

  /// A thread-safe `Result` type for provider outputs and errors.
  #[serde(serialize_with = "serialize_result")]
  pub result: Result<ProviderOutput, String>,
}

/// Reference to an active provider.
struct ProviderRef {
  /// Sender channel for sending inputs to the provider.
  async_input_tx: mpsc::Sender<ProviderInputMsg>,

  /// Sender channel for sending inputs to the provider.
  sync_input_tx: crossbeam::channel::Sender<ProviderInputMsg>,

  /// Handle to the provider's task.
  task_handle: task::JoinHandle<()>,

  /// Runtime type of the provider.
  runtime_type: RuntimeType,
}

/// Manages the creation and cleanup of providers.
pub struct ProviderManager {
  /// Handle to the Tauri application.
  app_handle: AppHandle,

  /// Map of active provider refs.
  provider_refs: Arc<Mutex<HashMap<String, ProviderRef>>>,

  /// Cache of provider emissions.
  emit_cache: Arc<Mutex<HashMap<String, ProviderEmission>>>,

  /// Sender channel for provider emissions.
  emit_tx: mpsc::UnboundedSender<ProviderEmission>,

  /// Shared `sysinfo` instance.
  sysinfo: Arc<Mutex<sysinfo::System>>,
}

impl ProviderManager {
  /// Creates a new provider manager.
  ///
  /// Returns a tuple containing the `ProviderManager` instance and a
  /// channel for provider emissions.
  pub fn new(
    app_handle: &AppHandle,
  ) -> (Arc<Self>, mpsc::UnboundedReceiver<ProviderEmission>) {
    let (emit_tx, emit_rx) = mpsc::unbounded_channel::<ProviderEmission>();

    (
      Arc::new(Self {
        app_handle: app_handle.clone(),
        provider_refs: Arc::new(Mutex::new(HashMap::new())),
        emit_cache: Arc::new(Mutex::new(HashMap::new())),
        sysinfo: Arc::new(Mutex::new(sysinfo::System::new_all())),
        emit_tx,
      }),
      emit_rx,
    )
  }

  /// Creates a provider with the given config.
  pub async fn create(
    &self,
    config_hash: String,
    config: ProviderConfig,
  ) -> anyhow::Result<()> {
    // If a provider with the given config already exists, re-emit its
    // latest emission and return early.
    {
      if let Some(found_emit) =
        self.emit_cache.lock().await.get(&config_hash)
      {
        tracing::info!(
          "Emitting cached provider emission for: {}",
          config_hash
        );

        self.app_handle.emit("provider-emit", found_emit)?;
        return Ok(());
      };
    }

    // Hold the lock for `provider_refs` to prevent duplicate providers
    // from potentially being created.
    let mut provider_refs = self.provider_refs.lock().await;

    // No-op if the provider has already been created (but has not emitted
    // yet). Multiple frontend clients can call `create` for the same
    // provider, and all will receive the same output once the provider
    // emits.
    if provider_refs.contains_key(&config_hash) {
      return Ok(());
    }

    tracing::info!("Creating provider: {}", config_hash);

    let (async_input_tx, async_input_rx) = mpsc::channel(1);
    let (sync_input_tx, sync_input_rx) = crossbeam::channel::bounded(1);

    let common = CommonProviderState {
      input: ProviderInput {
        async_rx: async_input_rx,
        sync_rx: sync_input_rx,
      },
      emitter: ProviderEmitter {
        emit_tx: self.emit_tx.clone(),
        config_hash: config_hash.clone(),
      },
      sysinfo: self.sysinfo.clone(),
    };

    let (task_handle, runtime_type) =
      self.create_instance(config, config_hash.clone(), common)?;

    let provider_ref = ProviderRef {
      async_input_tx,
      sync_input_tx,
      task_handle,
      runtime_type,
    };

    provider_refs.insert(config_hash, provider_ref);

    Ok(())
  }

  /// Creates a new provider instance.
  fn create_instance(
    &self,
    config: ProviderConfig,
    config_hash: String,
    common: CommonProviderState,
  ) -> anyhow::Result<(task::JoinHandle<()>, RuntimeType)> {
    let mut provider: Box<dyn Provider> = match config {
      #[cfg(windows)]
      ProviderConfig::Audio(config) => {
        Box::new(AudioProvider::new(config, common))
      }
      ProviderConfig::Battery(config) => {
        Box::new(BatteryProvider::new(config, common))
      }
      ProviderConfig::Cpu(config) => {
        Box::new(CpuProvider::new(config, common))
      }
      #[cfg(windows)]
      ProviderConfig::FocusedWindow(config) => {
        Box::new(FocusedWindowProvider::new(config, common))
      }
      ProviderConfig::Host(config) => {
        Box::new(HostProvider::new(config, common))
      }
      ProviderConfig::Ip(config) => {
        Box::new(IpProvider::new(config, common))
      }
      #[cfg(windows)]
      ProviderConfig::Komorebi(config) => {
        Box::new(KomorebiProvider::new(config, common))
      }
      #[cfg(windows)]
      ProviderConfig::Media(config) => {
        Box::new(MediaProvider::new(config, common))
      }
      ProviderConfig::Memory(config) => {
        Box::new(MemoryProvider::new(config, common))
      }
      ProviderConfig::Disk(config) => {
        Box::new(DiskProvider::new(config, common))
      }
      ProviderConfig::Network(config) => {
        Box::new(NetworkProvider::new(config, common))
      }
      ProviderConfig::Weather(config) => {
        Box::new(WeatherProvider::new(config, common))
      }
      #[cfg(windows)]
      ProviderConfig::Keyboard(config) => {
        Box::new(KeyboardProvider::new(config, common))
      }
      #[allow(unreachable_patterns)]
      _ => bail!("Provider not supported on this operating system."),
    };

    // Spawn the provider's task based on its runtime type.
    let runtime_type = provider.runtime_type();
    let task_handle = match &runtime_type {
      RuntimeType::Async => task::spawn(async move {
        provider.start_async().await;
        info!("Provider stopped: {}", config_hash);
      }),
      RuntimeType::Sync => task::spawn_blocking(move || {
        provider.start_sync();
        info!("Provider stopped: {}", config_hash);
      }),
    };

    Ok((task_handle, runtime_type))
  }

  /// Sends a function call through a channel to be executed by the
  /// provider.
  ///
  /// Returns the result of the function execution.
  pub async fn call_function(
    &self,
    config_hash: String,
    function: ProviderFunction,
  ) -> anyhow::Result<ProviderFunctionResponse> {
    info!(
      "Calling provider function: {:?} for: {}",
      function, config_hash
    );

    let provider_refs = self.provider_refs.lock().await;
    let provider_ref = provider_refs
      .get(&config_hash)
      .context("No provider found with config.")?;

    let (tx, rx) = oneshot::channel();
    match provider_ref.runtime_type {
      RuntimeType::Async => {
        provider_ref
          .async_input_tx
          .send(ProviderInputMsg::Function(function, tx))
          .await
          .context("Failed to send function call to provider.")?;
      }
      RuntimeType::Sync => {
        provider_ref
          .sync_input_tx
          .send(ProviderInputMsg::Function(function, tx))
          .context("Failed to send function call to provider.")?;
      }
    }

    rx.await?.map_err(anyhow::Error::msg)
  }

  /// Destroys and cleans up the provider with the given config.
  pub async fn stop(&self, config_hash: String) -> anyhow::Result<()> {
    let provider_ref = {
      let mut provider_refs = self.provider_refs.lock().await;

      // Evict the provider's emission from cache. Hold the lock for
      // `provider_refs` to avoid a race condition with provider
      // creation.
      let mut provider_cache = self.emit_cache.lock().await;
      let _ = provider_cache.remove(&config_hash);

      provider_refs
        .remove(&config_hash)
        .context("No provider found with config.")?
    };

    // Send shutdown signal to the provider.
    match provider_ref.runtime_type {
      RuntimeType::Async => {
        provider_ref
          .async_input_tx
          .send(ProviderInputMsg::Stop)
          .await
          .context("Failed to send shutdown signal to provider.")?;
      }
      RuntimeType::Sync => {
        provider_ref
          .sync_input_tx
          .send(ProviderInputMsg::Stop)
          .context("Failed to send shutdown signal to provider.")?;
      }
    }

    // Wait for the provider to stop.
    provider_ref.task_handle.await?;

    Ok(())
  }

  /// Updates the cache with the given provider emission.
  pub async fn update_cache(&self, emission: ProviderEmission) {
    let mut cache = self.emit_cache.lock().await;
    cache.insert(emission.config_hash.clone(), emission);
  }
}

/// Custom serializer for Result<ProviderOutput, String> that converts:
/// - Ok(output) -> {"output": output}
/// - Err(error) -> {"error": error}
fn serialize_result<S>(
  result: &Result<ProviderOutput, String>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let mut state = serializer.serialize_struct("Result", 1)?;

  match result {
    Ok(output) => state.serialize_field("output", output)?,
    Err(error) => state.serialize_field("error", error)?,
  }

  state.end()
}
