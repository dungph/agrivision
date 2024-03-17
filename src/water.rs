use std::{future::Future, time::Duration};

use embedded_hal::digital::OutputPin;
use gpio_cdev::LineRequestFlags;
use linux_embedded_hal::CdevPin;

use crate::config;

pub trait WaterIf {
    fn water(&mut self, dur: Duration) -> impl Future<Output = anyhow::Result<()>>;
}

impl WaterIf for Water {
    async fn water(&mut self, _dur: Duration) -> Result<(), anyhow::Error> {
        self.water().await
    }
}
impl WaterIf for DummyWater {
    async fn water(&mut self, dur: Duration) -> anyhow::Result<()> {
        async_std::task::sleep(dur).await;
        Ok(())
    }
}
pub struct DummyWater;

pub struct Water {
    pin: CdevPin,
}

impl Water {
    pub fn new(config: &config::Water) -> anyhow::Result<Self> {
        let pin = linux_embedded_hal::CdevPin::new(
            gpio_cdev::Chip::new(config.pin().chip())?
                .get_line(*config.pin().line())?
                .request(LineRequestFlags::OUTPUT, 0, "my_pin")?,
        )?;
        Ok(Self { pin })
    }
    pub async fn water(&mut self) -> anyhow::Result<()> {
        self.pin.set_high()?;
        async_std::task::sleep(Duration::from_millis(400)).await;
        self.pin.set_low()?;
        Ok(())
    }
}
