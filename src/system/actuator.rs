mod gpio_pin;
mod linear;
mod watering;

use std::time::Duration;

use derive_getters::Getters;
use futures::future::join;
use serde::{Deserialize, Serialize};

use {gpio_pin::LocalGpioConfig, linear::LocalLinearConfig, watering::WateringConfig};

#[derive(Getters, Serialize, Deserialize, Clone, Default)]
pub struct ActuatorProfile {
    pub en_pin: LocalGpioConfig,
    pub linear_x: LocalLinearConfig,
    pub linear_y: LocalLinearConfig,
    pub watering: WateringConfig,
}

impl ActuatorProfile {
    pub async fn goto(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
        log::info!("Moving");
        if x != 0 || y != 0 {
            self.en_pin.set_low()?;
        }
        let (_, _) = join(self.linear_x.goto(x), self.linear_y.goto(y)).await;

        if x == 0 && y == 0 {
            self.en_pin.set_high()?;
        }
        log::info!("Moving done");
        Ok(())
    }

    pub async fn water_at(&mut self, x: u32, y: u32, dur: Duration) -> anyhow::Result<()> {
        self.goto(x, y).await?;
        self.watering.water(dur).await?;
        Ok(())
    }
}
