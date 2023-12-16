use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sysinfo::{System, SystemExt};
use tokio::{sync::Mutex, task::AbortHandle};

use crate::providers::{
  interval_provider::IntervalProvider, variables::ProviderVariables,
};

use super::{MemoryProviderConfig, MemoryVariables};

pub struct MemoryProvider {
  pub config: Arc<MemoryProviderConfig>,
  abort_handle: Option<AbortHandle>,
  sysinfo: Arc<Mutex<System>>,
}

impl MemoryProvider {
  pub fn new(
    config: MemoryProviderConfig,
    sysinfo: Arc<Mutex<System>>,
  ) -> MemoryProvider {
    MemoryProvider {
      config: Arc::new(config),
      abort_handle: None,
      sysinfo,
    }
  }
}

#[async_trait]
impl IntervalProvider for MemoryProvider {
  type Config = MemoryProviderConfig;
  type State = Mutex<System>;

  fn refresh_interval_ms(&self) -> u64 {
    self.config.refresh_interval_ms
  }

  fn config(&self) -> Arc<MemoryProviderConfig> {
    self.config.clone()
  }

  fn state(&self) -> Arc<Mutex<System>> {
    self.sysinfo.clone()
  }

  fn abort_handle(&self) -> &Option<AbortHandle> {
    &self.abort_handle
  }

  fn set_abort_handle(&mut self, abort_handle: AbortHandle) {
    self.abort_handle = Some(abort_handle)
  }

  async fn get_refreshed_variables(
    _: &MemoryProviderConfig,
    sysinfo: &Mutex<System>,
  ) -> Result<ProviderVariables> {
    let mut sysinfo = sysinfo.lock().await;
    sysinfo.refresh_memory();

    Ok(ProviderVariables::Memory(MemoryVariables {
      free_memory: sysinfo.free_memory(),
      used_memory: sysinfo.used_memory(),
      total_memory: sysinfo.total_memory(),
      free_swap: sysinfo.free_swap(),
      used_swap: sysinfo.used_swap(),
      total_swap: sysinfo.total_swap(),
    }))
  }
}
