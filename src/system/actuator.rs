use std::time::Duration;

use derive_getters::Getters;
use futures_lite::future::zip;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use super::{
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
            self.en_pin.set_low();
        }
        let (_, _) = zip(self.linear_x.goto(x), self.linear_y.goto(y)).await;

        if x == 0 && y == 0 {
            self.en_pin.set_high()?;
        }
        log::info!("Moving done");
        Ok(())
    }
    pub async fn capture_at(&mut self, x: u32, y: u32) -> anyhow::Result<DynamicImage> {
        self.goto2(x, y).await;
        let ret = self.camera.capture().await?;
        Ok(ret)
    }

    pub async fn water_at(
        &mut self,
        x: u32,
        y: u32,
        dur: Duration,
    ) -> anyhow::Result<DynamicImage> {
        self.goto2(x, y).await?;
        self.watering.water(dur).await?;
        let ret = self.camera.capture().await?;
        Ok(ret)
    }
}

//pub struct Actuators {
//    en: CdevPin,
//    watering: Watering,
//    x: StepperLinear,
//    y: StepperLinear,
//    camera: camera::Camera,
//    config: LocalActuatorConfig,
//}
//
//impl Actuators {
//    pub fn new(config: &LocalActuatorConfig) -> anyhow::Result<Self> {
//        Ok(Self {
//            en: gpio_pin::get_output(config.en_pin())?,
//            watering: Watering::from(config.watering()),
//            x: StepperLinear::from(config.linear_x()),
//            y: StepperLinear::from(config.linear_y()),
//            camera: camera::Camera::from(config.camera()),
//            config: config.clone(),
//        })
//    }
//    pub fn current_config(&self) -> anyhow::Result<LocalActuatorConfig> {
//        Ok(self.config.clone())
//    }
//    pub async fn goto(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
//        log::info!("Moving");
//        if x != 0 || y != 0 {
//            self.en.set_value(0)?;
//        }
//        let (cx, cy) = futures_lite::future::zip(self.x.goto(x), self.y.goto(y)).await;
//        self.config.linear_x.position = cx?;
//        self.config.linear_y.position = cy?;
//
//        if x == 0 && y == 0 {
//            self.en.set_value(1)?;
//        }
//        log::info!("Moving done");
//        Ok(())
//    }
//    pub async fn water(&mut self, dur: Duration) -> anyhow::Result<()> {
//        log::info!("Moving");
//        self.watering.water(dur).await?;
//        log::info!("Moving done");
//        Ok(())
//    }
//    pub async fn capture(&mut self) -> anyhow::Result<DynamicImage> {
//        let image = self.camera.capture().await?;
//        Ok(image)
//    }
//}
//
//pub struct DummyActuators;
//impl DummyActuators {
//    pub fn new(config: &LocalActuatorConfig) -> anyhow::Result<Self> {
//        Ok(Self)
//    }
//    pub async fn goto(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
//        Ok(())
//    }
//    pub async fn water(&mut self, dur: Duration) -> anyhow::Result<()> {
//        Ok(())
//    }
//    pub async fn capture(&mut self) -> anyhow::Result<DynamicImage> {
//        Ok(DynamicImage::new(640, 640, ColorType::Rgb8))
//    }
//    pub async fn list_camera(&mut self) -> anyhow::Result<()> {
//        //let list = self.camera.list_camera().await?;
//        //self.subs.handle(list).await?;
//        Ok(())
//    }
//    pub async fn init_camera(&mut self, name: &Path) -> anyhow::Result<()> {
//        //self.camera.init_camera(name).await?;
//        Ok(())
//    }
//}
