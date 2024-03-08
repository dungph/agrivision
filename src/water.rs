use std::time::Duration;

use embedded_hal::digital::OutputPin;
use gpio_cdev::LineRequestFlags;
use linux_embedded_hal::CdevPin;

use crate::config;

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
