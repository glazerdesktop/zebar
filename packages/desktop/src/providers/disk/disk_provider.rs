use serde::{Deserialize, Serialize};
use sysinfo::Disks;

use crate::{
  common::{to_iec_bytes, to_si_bytes, SyncInterval},
  providers::{
    CommonProviderState, Provider, ProviderInputMsg, RuntimeType,
  },
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiskProviderConfig {
  pub refresh_interval: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskOutput {
  pub disks: Vec<Disk>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
  pub name: Option<String>,
  pub file_system: String,
  pub mount_point: String,
  pub total_space: DiskSizeMeasure,
  pub available_space: DiskSizeMeasure,
  pub is_removable: bool,
  pub drive_type: String,
}

pub struct DiskProvider {
  config: DiskProviderConfig,
  common: CommonProviderState,
  disks: Disks,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSizeMeasure {
  pub bytes: u64,
  pub si_value: f64,
  pub si_unit: String,
  pub iec_value: f64,
  pub iec_unit: String,
}

impl DiskProvider {
  pub fn new(
    config: DiskProviderConfig,
    common: CommonProviderState,
  ) -> DiskProvider {
    DiskProvider {
      config,
      common,
      disks: Disks::new_with_refreshed_list(),
    }
  }

  fn run_interval(&mut self) -> anyhow::Result<DiskOutput> {
    self.disks.refresh();

    let disks = self
      .disks
      .iter()
      .map(|disk| -> anyhow::Result<Disk> {
        let name = disk.name().to_string_lossy().to_string();

        Ok(Disk {
          name: (!name.is_empty()).then_some(name),
          file_system: disk.file_system().to_string_lossy().to_string(),
          mount_point: disk.mount_point().to_string_lossy().to_string(),
          total_space: Self::to_disk_size_measure(disk.total_space())?,
          available_space: Self::to_disk_size_measure(
            disk.available_space(),
          )?,
          is_removable: disk.is_removable(),
          drive_type: disk.kind().to_string(),
        })
      })
      .collect::<anyhow::Result<Vec<Disk>>>()?;

    Ok(DiskOutput { disks })
  }

  fn to_disk_size_measure(bytes: u64) -> anyhow::Result<DiskSizeMeasure> {
    let (si_value, si_unit) = to_si_bytes(bytes as f64);
    let (iec_value, iec_unit) = to_iec_bytes(bytes as f64);

    Ok(DiskSizeMeasure {
      bytes,
      si_value,
      si_unit,
      iec_value,
      iec_unit,
    })
  }
}

impl Provider for DiskProvider {
  fn runtime_type(&self) -> RuntimeType {
    RuntimeType::Sync
  }

  fn start_sync(&mut self) {
    let mut interval = SyncInterval::new(self.config.refresh_interval);

    loop {
      crossbeam::select! {
        recv(interval.tick()) -> _ => {
          let output = self.run_interval();
          self.common.emitter.emit_output(output);
        }
        recv(self.common.input.sync_rx) -> input => {
          if let Ok(ProviderInputMsg::Stop) = input {
            break;
          }
        }
      }
    }
  }
}
