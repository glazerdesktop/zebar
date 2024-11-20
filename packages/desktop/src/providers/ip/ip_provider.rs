use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::ipinfo_res::IpinfoRes;
use crate::{
  common::AsyncInterval,
  providers::{
    CommonProviderState, Provider, ProviderInputMsg, RuntimeType,
  },
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IpProviderConfig {
  pub refresh_interval: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpOutput {
  pub address: String,
  pub approx_city: String,
  pub approx_country: String,
  pub approx_latitude: f32,
  pub approx_longitude: f32,
}

pub struct IpProvider {
  config: IpProviderConfig,
  common: CommonProviderState,
  http_client: Client,
}

impl IpProvider {
  pub fn new(
    config: IpProviderConfig,
    common: CommonProviderState,
  ) -> IpProvider {
    IpProvider {
      config,
      common,
      http_client: Client::new(),
    }
  }

  async fn run_interval(&mut self) -> anyhow::Result<IpOutput> {
    Self::query_ip(&self.http_client).await
  }

  pub async fn query_ip(http_client: &Client) -> anyhow::Result<IpOutput> {
    let res = http_client
      .get("https://ipinfo.io/json")
      .send()
      .await?
      .json::<IpinfoRes>()
      .await?;

    let mut loc_parts = res.loc.split(',');

    Ok(IpOutput {
      address: res.ip,
      approx_city: res.city,
      approx_country: res.country,
      approx_latitude: loc_parts
        .next()
        .and_then(|lat| lat.parse::<f32>().ok())
        .context("Failed to parse latitude from IPinfo.")?,
      approx_longitude: loc_parts
        .next()
        .and_then(|long| long.parse::<f32>().ok())
        .context("Failed to parse longitude from IPinfo.")?,
    })
  }
}

#[async_trait]
impl Provider for IpProvider {
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
