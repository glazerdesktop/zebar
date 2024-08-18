use std::sync::Arc;

use async_trait::async_trait;
use sysinfo::System;
use tokio::{sync::Mutex, task::AbortHandle};

use super::{CpuProviderConfig, CpuVariables};
use crate::providers::{
  provider::IntervalProvider, variables::ProviderVariables,
};

pub struct CpuProvider {
  pub config: Arc<CpuProviderConfig>,
  abort_handle: Option<AbortHandle>,
  sysinfo: Arc<Mutex<System>>,
}

impl CpuProvider {
  pub fn new(
    config: CpuProviderConfig,
    sysinfo: Arc<Mutex<System>>,
  ) -> CpuProvider {
    CpuProvider {
      config: Arc::new(config),
      abort_handle: None,
      sysinfo,
    }
  }
}

#[async_trait]
impl IntervalProvider for CpuProvider {
  type Config = CpuProviderConfig;
  type State = Mutex<System>;

  fn config(&self) -> Arc<CpuProviderConfig> {
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
    _: &CpuProviderConfig,
    sysinfo: &Mutex<System>,
  ) -> anyhow::Result<ProviderVariables> {
    let mut sysinfo = sysinfo.lock().await;
    sysinfo.refresh_cpu();

    Ok(ProviderVariables::Cpu(CpuVariables {
      usage: sysinfo.global_cpu_info().cpu_usage(),
      frequency: sysinfo.global_cpu_info().frequency(),
      logical_core_count: sysinfo.cpus().len(),
      physical_core_count: sysinfo
        .physical_core_count()
        .unwrap_or(sysinfo.cpus().len()),
      vendor: sysinfo.global_cpu_info().vendor_id().into(),
    }))
  }
}
