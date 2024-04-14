use std::{error::Error, time::Duration};

use derive_getters::Getters;
use embedded_hal::digital::OutputPin;
use linux_embedded_hal::CdevPin;
use serde::{Deserialize, Serialize};

use super::gpio_pin::{get_output, LocalGpioConfig};

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

//pub struct Watering {
//    pin: Option<CdevPin>,
//    pub pin_conf: LocalGpioConfig,
//}
//
//impl From<&WateringConfig> for Watering {
//    fn from(value: &WateringConfig) -> Self {
//        Self::from(value.clone())
//    }
//}
//impl From<WateringConfig> for Watering {
//    fn from(value: WateringConfig) -> Self {
//        Self {
//            pin: None,
//            pin_conf: value.pin,
//        }
//    }
//}
//
//impl Watering {
//    //pub fn new(config: &WateringConfig) -> anyhow::Result<Self> {
//    //    let pin = get_output(&config.pin)?;
//    //    Ok(Self {
//    //        pin: None,
//    //        pin_conf: config.pin,
//    //    })
//    //}
//    pub async fn water(&mut self, dur: Duration) -> anyhow::Result<()> {
//        let mut pin = loop {
//            if let Some(pin) = self.pin.take() {
//                break pin;
//            } else {
//                self.pin.replace(get_output(&self.pin_conf)?);
//            };
//        };
//        pin.set_high()?;
//        async_std::task::sleep(dur).await;
//        pin.set_low()?;
//
//        self.pin.replace(pin);
//        Ok(())
//    }
//}
