use serde::Serialize;

#[cfg(windows)]
use super::{
  audio::AudioOutput, focused_window::FocusedWindowOutput,
  keyboard::KeyboardOutput, komorebi::KomorebiOutput, media::MediaOutput,
};
use super::{
  battery::BatteryOutput, cpu::CpuOutput, disk::DiskOutput,
  host::HostOutput, ip::IpOutput, memory::MemoryOutput,
  network::NetworkOutput, weather::WeatherOutput,
};

/// Implements `From<T>` for `ProviderOutput` for each given variant.
macro_rules! impl_provider_output {
  ($($variant:ident($type:ty)),* $(,)?) => {
    $(
      impl From<$type> for ProviderOutput {
        fn from(value: $type) -> Self {
          Self::$variant(value)
        }
      }
    )*
  };
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ProviderOutput {
  #[cfg(windows)]
  Audio(AudioOutput),
  Battery(BatteryOutput),
  Cpu(CpuOutput),
  #[cfg(windows)]
  FocusedWindow(FocusedWindowOutput),
  Host(HostOutput),
  Ip(IpOutput),
  #[cfg(windows)]
  Komorebi(KomorebiOutput),
  #[cfg(windows)]
  Media(MediaOutput),
  Memory(MemoryOutput),
  Disk(DiskOutput),
  Network(NetworkOutput),
  Weather(WeatherOutput),
  #[cfg(windows)]
  Keyboard(KeyboardOutput),
}

impl_provider_output! {
  Battery(BatteryOutput),
  Cpu(CpuOutput),
  Host(HostOutput),
  Ip(IpOutput),
  Memory(MemoryOutput),
  Disk(DiskOutput),
  Network(NetworkOutput),
  Weather(WeatherOutput)
}

#[cfg(windows)]
impl_provider_output! {
  Audio(AudioOutput),
  FocusedWindow(FocusedWindowOutput),
  Komorebi(KomorebiOutput),
  Media(MediaOutput),
  Keyboard(KeyboardOutput)
}
