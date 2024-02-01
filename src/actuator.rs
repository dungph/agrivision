mod linear;
mod stepper;
mod switcher;

use std::time::Duration;

use async_std::{sync::Mutex, task::JoinHandle};
use once_cell::sync::Lazy;

use self::{linear::StepperLinear, switcher::Switcher};

pub async fn start_pump(pin: u8, on: Duration, off: Duration) -> anyhow::Result<()> {
    static PUMP_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
    if let Some(handle) = PUMP_TASK.lock().await.take() {
        handle.cancel().await;
    }
    let gpio = rppal::gpio::Gpio::new()?.get(pin)?;

    let task = async_std::task::spawn(async move {
        let mut switch = Switcher::new(gpio.into_output());
        loop {
            switch.on();
            async_std::task::sleep(on).await;
            switch.off();
            async_std::task::sleep(off).await;
        }
    });
    PUMP_TASK.lock().await.replace(task);
    Ok(())
}

//static X_ACTUATORS: Mutex<Option<StepperLinear>> = Mutex::new(None);
type StaticPair<T> = Lazy<async_channel::Sender<T>, async_channel::Receiver<T>>;
static LINEAR_ACTUATORS = 

pub async fn start_linear() {
    static HTTP_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
    if let Some(handle) = HTTP_TASK.lock().await.take() {
        handle.cancel().await;
    }

    let task = async_std::task::spawn(async move {
        let config = crate::settings::linear_actuators();
        let gpio = rppal::gpio::Gpio::new().unwrap();
        let mut enn = gpio.get(config.en_pin as u8).unwrap().into_output();
        let dir_x = gpio.get(config.x.dir_pin as u8).unwrap().into_output_low();
        let step_x = gpio.get(config.x.step_pin as u8).unwrap().into_output_low();
        let dir_y = gpio.get(config.y.dir_pin as u8).unwrap().into_output_low();
        let step_y = gpio.get(config.y.step_pin as u8).unwrap().into_output_low();
        let dir_z = gpio.get(config.z.dir_pin as u8).unwrap().into_output_low();
        let step_z = gpio.get(config.z.step_pin as u8).unwrap().into_output_low();
        let mut linear_x =
            StepperLinear::new(stepper::Stepper::new(config.x.reversed, dir_x, step_x));
        let mut linear_y =
            StepperLinear::new(stepper::Stepper::new(config.y.reversed, dir_y, step_y));
        let mut linear_z =
            StepperLinear::new(stepper::Stepper::new(config.z.reversed, dir_z, step_z));
        loop {
            enn.set_low();
            linear_x.r#move(40).await;
            linear_x.r#move(-40).await;
            linear_y.r#move(40).await;
            linear_y.r#move(-40).await;
            linear_z.r#move(40).await;
            linear_z.r#move(-40).await;
            enn.set_high();
            async_std::task::sleep(Duration::from_secs(2)).await;
        }
    });

    HTTP_TASK.lock().await.replace(task);
}

//pub async fn set_stepper_actuator(step_pins: [Pin; 3], dir_pin: [Pin; 3]) {
//    let [s1, s2, s3] = step_pins;
//    let [d1, d2, d3] = dir_pin;
//    let steppers = [
//        Stepper::new(s1.into_output(), d1.into_output()),
//        Stepper::new(s2.into_output(), d2.into_output()),
//        Stepper::new(s3.into_output(), d3.into_output()),
//    ];
//    ACTUATORS
//        .lock()
//        .await
//        .replace(steppers.map(StepperLinear::new));
//}
//pub async fn goto(x: i32, y: i32, z: i32) -> Option<(i32, i32, i32)> {
//    if let Some([xc, yc, zc]) = ACTUATORS.lock().await.as_mut() {
//        let x = xc.goto(x).await;
//        let y = yc.goto(y).await;
//        let z = zc.goto(z).await;
//        Some((x, y, z))
//    } else {
//        None
//    }
//}
//
//pub async fn r#move(x: i32, y: i32, z: i32) -> Option<(i32, i32, i32)> {
//    if let Some([xc, yc, zc]) = ACTUATORS.lock().await.as_mut() {
//        let x = xc.r#move(x).await;
//        let y = yc.r#move(y).await;
//        let z = zc.r#move(z).await;
//        Some((x, y, z))
//    } else {
//        None
//    }
//}
