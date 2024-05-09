use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use derive_getters::Getters;
use embedded_hal::digital::OutputPin;
use gpio_cdev::LineRequestFlags;
use linux_embedded_hal::CdevPin;
use serde::{Deserialize, Serialize};

type Gpios = Mutex<BTreeMap<(PathBuf, u32), Arc<Mutex<CdevPin>>>>;
static GPIOS: Gpios = Mutex::new(BTreeMap::new());

pub fn get_output(pin: &LocalGpioConfig) -> anyhow::Result<Arc<Mutex<CdevPin>>> {
    let mut gpios = GPIOS.lock().unwrap();

    if let Some(pin) = gpios.get(&(pin.chip.clone(), pin.line)) {
        Ok(pin.clone())
    } else {
        let gpin = CdevPin::new(
            gpio_cdev::Chip::new(pin.chip())?
                .get_line(*pin.line())?
                .request(LineRequestFlags::OUTPUT, 0, "my_pin")?,
        )?;

        let gpin = Arc::new(Mutex::new(gpin));
        gpios.insert((pin.chip.clone(), pin.line), gpin.clone());
        Ok(gpin)
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct LocalGpioConfig {
    pub chip: PathBuf,
    pub line: u32,
}

impl Default for LocalGpioConfig {
    fn default() -> Self {
        Self {
            chip: "/dev/gpiochip0".into(),
            line: 0,
        }
    }
}

impl LocalGpioConfig {
    pub fn set_high(&mut self) -> anyhow::Result<()> {
        if self.chip == PathBuf::from("stub") {
            return Ok(());
        }
        let pin = get_output(self)?;
        pin.lock().unwrap().set_high()?;
        Ok(())
    }
    pub fn set_low(&mut self) -> anyhow::Result<()> {
        if self.chip == PathBuf::from("stub") {
            return Ok(());
        }
        let pin = get_output(self)?;
        pin.lock().unwrap().set_low()?;
        Ok(())
    }
}
