use rppal::gpio::OutputPin;
pub struct Switcher {
    pin: OutputPin,
}

impl Switcher {
    pub fn new(pin: OutputPin) -> Self {
        Self { pin }
    }

    pub fn on(&mut self) {
        self.pin.set_high();
    }

    pub fn off(&mut self) {
        self.pin.set_low();
    }
}
