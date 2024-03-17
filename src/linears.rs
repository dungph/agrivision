use std::future::Future;
use std::time::Duration;

use async_std::task::sleep;
use embedded_hal::digital::OutputPin;
use gpio_cdev::LineRequestFlags;
use linux_embedded_hal::CdevPin;

use crate::config::Config;

pub trait Linear2D {
    fn goto(&mut self, x: u32, y: u32) -> impl Future<Output = anyhow::Result<()>>;
}

impl Linear2D for Linears {
    async fn goto(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
        self.goto(x as i32, y as i32).await
    }
}

impl Linear2D for DummyLinear2D {
    async fn goto(&mut self, _x: u32, _y: u32) -> anyhow::Result<()> {
        async_std::task::sleep(Duration::from_secs(2)).await;
        Ok(())
    }
}

pub struct DummyLinear2D;

pub struct Linears {
    x: StepperLinear,
    y: StepperLinear,
}

impl Linears {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let x = StepperLinear::new(config.linear_x())?;
        let y = StepperLinear::new(config.linear_y())?;
        Ok(Self { x, y })
    }
    pub async fn goto(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        let (_, _) = futures_lite::future::zip(self.x.goto(x), self.y.goto(y)).await;
        Ok(())
    }
}

use crate::config;

fn get_output(pin: &config::GpioPin) -> anyhow::Result<linux_embedded_hal::CdevPin> {
    Ok(linux_embedded_hal::CdevPin::new(
        gpio_cdev::Chip::new(pin.chip())?
            .get_line(*pin.line())?
            .request(LineRequestFlags::OUTPUT, 0, "my_pin")?,
    )?)
}

pub struct StepperLinear {
    en_pin: CdevPin,
    dir_pin: CdevPin,
    step_pin: CdevPin,
    reversed: bool,
    pub position: i32,
    pub min_speed: u32,
    pub max_speed: u32,
    pub accelerate: u32,
    pub steps_per_mm: u32,
}

impl StepperLinear {
    pub fn new(settings: &config::Linear) -> anyhow::Result<Self> {
        let en_pin = get_output(settings.en_pin())?;
        let dir_pin = get_output(settings.dir_pin())?;
        let step_pin = get_output(settings.step_pin())?;
        Ok(Self {
            en_pin,
            dir_pin,
            step_pin,
            reversed: *settings.reverse(),
            position: 0,
            min_speed: *settings.min_mm_per_s(),
            max_speed: *settings.max_mm_per_s(),
            accelerate: *settings.accelerate(),
            steps_per_mm: *settings.step_per_ms(),
        })
    }

    pub async fn forward(&mut self) -> anyhow::Result<()> {
        if self.reversed {
            self.dir_pin.set_high()?;
        } else {
            self.dir_pin.set_low()?;
        }
        sleep(Duration::from_micros(10)).await;
        Ok(())
    }

    pub async fn backward(&mut self) -> anyhow::Result<()> {
        if self.reversed {
            self.dir_pin.set_low()?;
        } else {
            self.dir_pin.set_high()?;
        }
        sleep(Duration::from_micros(10)).await;
        Ok(())
    }

    pub async fn accel_move(
        &mut self,
        step: u32,
        min_sps: u32,
        max_sps: u32,
        accel: u32,
    ) -> anyhow::Result<()> {
        let mut sps = min_sps as f32;
        let mut step_count = 0;
        let mid_step = step / 2;
        while (sps < max_sps as f32) && step_count < mid_step {
            let period = 1f32 / sps;
            self.step_pin.set_high()?;
            sleep(Duration::from_micros(2)).await;
            self.step_pin.set_low()?;
            sleep(Duration::from_micros((period * 1_000_000f32) as u64)).await;
            sps += accel as f32 * period;
            step_count += 1;
        }

        if step_count < mid_step {
            let period = 1f32 / sps;
            for _ in 0..step_count * 2 {
                self.step_pin.set_high()?;
                sleep(Duration::from_micros(2)).await;
                self.step_pin.set_low()?;
                sleep(Duration::from_micros((period * 1_000_000f32) as u64)).await;
                step_count += 1;
            }
        }
        while step_count < step {
            let period = 1f32 / sps;
            self.step_pin.set_high()?;
            sleep(Duration::from_micros(2)).await;
            self.step_pin.set_low()?;
            sleep(Duration::from_micros((period * 1_000_000f32) as u64)).await;
            sps -= accel as f32 * period;
            step_count += 1;
        }
        Ok(())
    }
    pub async fn goto(&mut self, position: i32) -> anyhow::Result<i32> {
        self.en_pin.set_low()?;
        let diff = position - self.position;
        self.forward().await?;
        if diff < 0 {
            self.backward().await?;
        }
        let steps = diff.unsigned_abs() * self.steps_per_mm;
        let min_sps = self.min_speed * self.steps_per_mm;
        let max_sps = self.max_speed * self.steps_per_mm;
        let accel = self.accelerate * self.steps_per_mm;

        self.accel_move(steps, min_sps, max_sps, accel).await?;
        self.position += diff;
        if self.position == 0 {
            self.en_pin.set_high()?;
        }
        Ok(self.position)
    }

    pub async fn r#move(&mut self, length: i32) -> anyhow::Result<i32> {
        self.goto(self.position + length).await
    }
}
