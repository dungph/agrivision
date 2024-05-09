use std::time::Duration;

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use super::gpio_pin::LocalGpioConfig;

#[derive(Getters, Serialize, Deserialize, Debug, Clone, Default)]
pub struct WateringConfig {
    pub pin: LocalGpioConfig,
}

impl WateringConfig {
    pub async fn water(&mut self, dur: Duration) -> anyhow::Result<()> {
        self.pin.set_high()?;
        async_std::task::sleep(dur).await;
        self.pin.set_low()?;
        Ok(())
    }
}
