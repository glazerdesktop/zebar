use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use tokio::task::AbortHandle;

use crate::providers::{
  interval_provider::IntervalProvider, variables::ProviderVariables,
};

use super::{
  open_meteo_res::OpenMeteoRes, WeatherProviderConfig, WeatherStatus,
  WeatherVariables,
};

pub struct WeatherProvider {
  pub config: Arc<WeatherProviderConfig>,
  abort_handle: Option<AbortHandle>,
  http_client: Arc<Client>,
}

impl WeatherProvider {
  pub fn new(config: WeatherProviderConfig) -> WeatherProvider {
    WeatherProvider {
      config: Arc::new(config),
      abort_handle: None,
      http_client: Arc::new(Client::new()),
    }
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
      1 | 2 => match is_daytime {
        true => WeatherStatus::CloudyDay,
        false => WeatherStatus::CloudyNight,
      },
      3..=50 => WeatherStatus::Overcast,
      51..=62 => WeatherStatus::LightRain,
      63..=70 => WeatherStatus::HeavyRain,
      71..=79 => WeatherStatus::Snow,
      80..=84 => WeatherStatus::HeavyRain,
      85..=94 => WeatherStatus::Snow,
      95..=u32::MAX => WeatherStatus::Snow,
    }
  }
}

#[async_trait]
impl IntervalProvider for WeatherProvider {
  type Config = WeatherProviderConfig;
  type State = Client;

  fn config(&self) -> Arc<WeatherProviderConfig> {
    self.config.clone()
  }

  fn state(&self) -> Arc<Client> {
    self.http_client.clone()
  }

  fn abort_handle(&self) -> &Option<AbortHandle> {
    &self.abort_handle
  }

  fn set_abort_handle(&mut self, abort_handle: AbortHandle) {
    self.abort_handle = Some(abort_handle)
  }

  async fn get_refreshed_variables(
    config: &WeatherProviderConfig,
    http_client: &Client,
  ) -> Result<ProviderVariables> {
    let res = http_client
      .get("https://api.open-meteo.com/v1/forecast")
      .query(&[
        ("temperature_unit", "celsius"),
        ("latitude", &config.latitude.to_string()),
        ("longitude", &config.longitude.to_string()),
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

    Ok(ProviderVariables::Weather(WeatherVariables {
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
    }))
  }
}
