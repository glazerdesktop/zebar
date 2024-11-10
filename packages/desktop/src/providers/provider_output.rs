use serde::Serialize;

use super::{
  audio::AudioOutput, battery::BatteryOutput, cpu::CpuOutput,
  disk::DiskOutput, host::HostOutput, ip::IpOutput, memory::MemoryOutput,
  network::NetworkOutput, weather::WeatherOutput,
};
#[cfg(windows)]
use super::{keyboard::KeyboardOutput, komorebi::KomorebiOutput};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ProviderOutput {
  Audio(AudioOutput),
  Battery(BatteryOutput),
  Cpu(CpuOutput),
  Host(HostOutput),
  Ip(IpOutput),
  #[cfg(windows)]
  Komorebi(KomorebiOutput),
  Memory(MemoryOutput),
  Disk(DiskOutput),
  Network(NetworkOutput),
  Weather(WeatherOutput),
  #[cfg(windows)]
  Keyboard(KeyboardOutput),
}
