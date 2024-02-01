use std::time::Duration;

use async_std::task::sleep;
use rppal::gpio::OutputPin;

pub struct Stepper {
    reversed: bool,
    dir_pin: OutputPin,
    step_pin: OutputPin,
}

impl Stepper {
    pub fn new(reversed: bool, dir_pin: OutputPin, step_pin: OutputPin) -> Self {
        Self {
            reversed,
            dir_pin,
            step_pin,
        }
    }

    pub async fn forward(&mut self) {
        if self.reversed {
            self.dir_pin.set_high();
        } else {
            self.dir_pin.set_low();
        }
        sleep(Duration::from_micros(10)).await;
    }

    pub async fn backward(&mut self) {
        if self.reversed {
            self.dir_pin.set_low();
        } else {
            self.dir_pin.set_high();
        }
        sleep(Duration::from_micros(10)).await;
    }

    pub async fn accel_move(&mut self, step: u32, min_sps: u32, max_sps: u32, accel: u32) {
        let mut sps = min_sps as f32;
        let mut step_count = 0;
        let mid_step = step / 2;
        while (sps < max_sps as f32) && step_count < mid_step {
            let period = 1f32 / sps;
            self.step_pin.set_high();
            sleep(Duration::from_micros(10)).await;
            self.step_pin.set_low();
            sleep(Duration::from_micros((period * 1_000_000f32) as u64)).await;
            sps += accel as f32 * period;
            step_count += 1;
        }

        if step_count < mid_step {
            let period = 1f32 / sps;
            for _ in 0..step_count * 2 {
                self.step_pin.set_high();
                sleep(Duration::from_micros(1)).await;
                self.step_pin.set_low();
                sleep(Duration::from_micros((period * 1_000_000f32) as u64)).await;
                step_count += 1;
            }
        }
        while step_count < step {
            let period = 1f32 / sps;
            self.step_pin.set_high();
            sleep(Duration::from_micros(10)).await;
            self.step_pin.set_low();
            sleep(Duration::from_micros((period * 1_000_000f32) as u64)).await;
            sps -= accel as f32 * period;
            step_count += 1;
        }
    }
}
