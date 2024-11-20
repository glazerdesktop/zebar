use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use super::ProviderResult;

#[async_trait]
pub trait Provider: Send + Sync {
  fn threading_type(&self) -> ThreadingType;

  /// Callback for when the provider is started.
  async fn run_async(&self, _emit_result_tx: Sender<ProviderResult>) {
    // TODO: Change to not implemented exception.
    todo!()
  }

  /// Callback for when the provider is started.
  fn run_sync(&self, _emit_result_tx: Sender<ProviderResult>) {
    // TODO: Change to not implemented exception.
    todo!()
  }

  /// Callback for when the provider is stopped.
  async fn on_stop(&self) {
    // No-op by default.
  }
}

/// Determines whether `run_sync` or `run_async` is called.`
pub enum ThreadingType {
  Sync,
  Async,
}

/// Implements the `Provider` trait for the given struct.
///
/// Expects that the struct has a `refresh_interval_ms` and `run_interval`
/// method.
#[macro_export]
macro_rules! impl_interval_provider {
  ($type:ty, $allow_identical_emits:expr) => {
    #[async_trait::async_trait]
    impl crate::providers::Provider for $type {
      fn threading_type(&self) -> crate::providers::ThreadingType {
        crate::providers::ThreadingType::Sync
      }

      async fn run_async(
        &self,
        emit_result_tx: tokio::sync::mpsc::Sender<
          crate::providers::ProviderResult,
        >,
      ) {
        let mut interval = tokio::time::interval(
          std::time::Duration::from_millis(self.refresh_interval_ms()),
        );

        // Skip missed ticks when the interval runs. This prevents a burst
        // of backlogged ticks after a delay.
        interval
          .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut last_interval_res: Option<
          crate::providers::ProviderResult,
        > = None;

        loop {
          interval.tick().await;

          let interval_res = self.run_interval().await.into();

          if $allow_identical_emits
            || last_interval_res.as_ref() != Some(&interval_res)
          {
            let send_res = emit_result_tx.send(interval_res.clone()).await;

            if let Err(err) = send_res {
              tracing::error!("Error sending provider result: {:?}", err);
            }

            last_interval_res = Some(interval_res);
          }
        }
      }
    }
  };
}
