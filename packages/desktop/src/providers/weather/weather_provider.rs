use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::open_meteo_res::OpenMeteoRes;
use crate::{
  common::AsyncInterval,
  providers::{
    ip::IpProvider, CommonProviderState, Provider, ProviderInputMsg,
    RuntimeType,
  },
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WeatherProviderConfig {
  pub refresh_interval: u64,
  pub latitude: Option<f32>,
  pub longitude: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherOutput {
  pub is_daytime: bool,
  pub status: WeatherStatus,
  pub celsius_temp: f32,
  pub fahrenheit_temp: f32,
  pub wind_speed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherStatus {
  ClearDay,
  ClearNight,
  CloudyDay,
  CloudyNight,
  LightRainDay,
  LightRainNight,
  HeavyRainDay,
  HeavyRainNight,
  SnowDay,
  SnowNight,
  ThunderDay,
  ThunderNight,
}

pub struct WeatherProvider {
  config: WeatherProviderConfig,
  common: CommonProviderState,
  http_client: Client,
}

impl WeatherProvider {
  pub fn new(
    config: WeatherProviderConfig,
    common: CommonProviderState,
  ) -> WeatherProvider {
    WeatherProvider {
      config,
      common,
      http_client: Client::new(),
    }
  }

  async fn run_interval(&self) -> anyhow::Result<WeatherOutput> {
    let (latitude, longitude) = {
      match (self.config.latitude, self.config.longitude) {
        (Some(lat), Some(lon)) => (lat, lon),
        _ => {
          let ip_output = IpProvider::query_ip(&self.http_client).await?;
          (ip_output.approx_latitude, ip_output.approx_longitude)
        }
      }
    };

    let res = self
      .http_client
      .get("https://api.open-meteo.com/v1/forecast")
      .query(&[
        ("temperature_unit", "celsius"),
        ("latitude", &latitude.to_string()),
        ("longitude", &longitude.to_string()),
        ("current_weather", "true"),
        ("daily", "sunset,sunrise"),
        ("timezone", "auto"),
      ])
      .send()
      .await?
      .json::<OpenMeteoRes>()
      .await?;

    let current_weather = res.current_weather;
    let is_daytime = current_weather.is_day == 1;

    Ok(WeatherOutput {
      is_daytime,
      status: Self::get_weather_status(
        current_weather.weather_code,
        is_daytime,
      ),
      celsius_temp: current_weather.temperature,
      fahrenheit_temp: Self::celsius_to_fahrenheit(
        current_weather.temperature,
      ),
      wind_speed: current_weather.wind_speed,
    })
  }

  fn celsius_to_fahrenheit(celsius_temp: f32) -> f32 {
    return (celsius_temp * 9.) / 5. + 32.;
  }

  /// Relevant documentation: https://open-meteo.com/en/docs#weathervariables
  fn get_weather_status(code: u32, is_daytime: bool) -> WeatherStatus {
    match code {
      0 => match is_daytime {
        true => WeatherStatus::ClearDay,
        false => WeatherStatus::ClearNight,
      },
      1..=50 => match is_daytime {
        true => WeatherStatus::CloudyDay,
        false => WeatherStatus::CloudyNight,
      },
      51..=62 => match is_daytime {
        true => WeatherStatus::LightRainDay,
        false => WeatherStatus::LightRainNight,
      },
      63..=70 => match is_daytime {
        true => WeatherStatus::HeavyRainDay,
        false => WeatherStatus::HeavyRainNight,
      },
      71..=79 => match is_daytime {
        true => WeatherStatus::SnowDay,
        false => WeatherStatus::SnowNight,
      },
      80..=84 => match is_daytime {
        true => WeatherStatus::HeavyRainDay,
        false => WeatherStatus::HeavyRainNight,
      },
      85..=94 => match is_daytime {
        true => WeatherStatus::SnowDay,
        false => WeatherStatus::SnowNight,
      },
      95..=u32::MAX => match is_daytime {
        true => WeatherStatus::ThunderDay,
        false => WeatherStatus::ThunderNight,
      },
    }
  }
}

#[async_trait]
impl Provider for WeatherProvider {
  fn runtime_type(&self) -> RuntimeType {
    RuntimeType::Async
  }

  async fn start_async(&mut self) {
    let mut interval = AsyncInterval::new(self.config.refresh_interval);

    loop {
      tokio::select! {
        _ = interval.tick() => {
          let output = self.run_interval().await;
          self.common.emitter.emit_output(output);
        }
        Some(message) = self.common.input.async_rx.recv() => {
          if let ProviderInputMsg::Stop = message {
            break;
          }
        }
      }
    }
  }
}
