use serde::Serialize;

use super::{
  battery::BatteryOutput, cpu::CpuOutput, host::HostOutput, ip::IpOutput,
  memory::MemoryOutput, network::NetworkOutput, weather::WeatherOutput,
};
#[cfg(windows)]
use super::{komorebi::KomorebiOutput, language::LanguageVariables};

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ProviderOutput {
  Battery(BatteryOutput),
  Cpu(CpuOutput),
  Host(HostOutput),
  Ip(IpOutput),
  #[cfg(windows)]
  Komorebi(KomorebiOutput),
  Memory(MemoryOutput),
  Network(NetworkOutput),
  Weather(WeatherOutput),
  #[cfg(windows)]
  Language(LanguageVariables),
}
