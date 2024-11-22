#[cfg(windows)]
mod audio;
mod battery;
mod cpu;
mod disk;
mod focused_window;
mod host;
mod ip;
#[cfg(windows)]
mod keyboard;
#[cfg(windows)]
mod komorebi;
#[cfg(windows)]
mod media;
mod memory;
mod network;
mod provider;
mod provider_config;
mod provider_function;
mod provider_manager;
mod provider_output;
mod weather;

pub use provider::*;
pub use provider_config::*;
pub use provider_function::*;
pub use provider_manager::*;
pub use provider_output::*;
