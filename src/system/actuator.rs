mod camera;
mod gpio_pin;
mod linear;
mod watering;

use std::time::Duration;

use derive_getters::Getters;
use futures_lite::future::zip;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use {
    camera::CameraConfig, gpio_pin::LocalGpioConfig, linear::LocalLinearConfig,
    watering::WateringConfig,
};

#[derive(Getters, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActuatorProfile {
    pub en_pin: LocalGpioConfig,
    pub linear_x: LocalLinearConfig,
    pub linear_y: LocalLinearConfig,
    pub watering: WateringConfig,
    pub camera: CameraConfig,
}

impl ActuatorProfile {
    pub async fn goto2(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
        log::info!("Moving");
        if x != 0 || y != 0 {
            self.en_pin.set_low()?;
        }
        let (_, _) = zip(self.linear_x.goto(x), self.linear_y.goto(y)).await;

        if x == 0 && y == 0 {
            self.en_pin.set_high()?;
        }
        log::info!("Moving done");
        Ok(())
    }
    pub async fn capture_at(&mut self, x: u32, y: u32) -> anyhow::Result<DynamicImage> {
        self.goto2(x, y).await.map_err(|e| dbg!(e))?;
        let ret = self.camera.capture().await.map_err(|e| dbg!(e))?;
        Ok(ret)
    }

    pub async fn water_at(&mut self, x: u32, y: u32, dur: Duration) -> anyhow::Result<()> {
        self.goto2(x, y).await?;
        self.watering.water(dur).await?;
        Ok(())
    }
}
