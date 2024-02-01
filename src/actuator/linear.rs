use super::stepper::Stepper;

pub struct StepperLinear {
    stepper: Stepper,
    pub position: i32,
    pub min_speed: u32,
    pub max_speed: u32,
    pub accelerate: u32,
    pub step_per_mm: u32,
}

impl StepperLinear {
    pub fn new(stepper: Stepper) -> Self {
        Self {
            accelerate: 20,
            position: 0,
            min_speed: 5,
            max_speed: 200,
            step_per_mm: 40,
            stepper,
        }
    }

    pub async fn goto(&mut self, position: i32) -> i32 {
        let diff = position - self.position;
        self.stepper.forward().await;
        if diff < 0 {
            self.stepper.backward().await;
        }
        let steps = diff.unsigned_abs() * self.step_per_mm;
        let min_sps = self.min_speed * self.step_per_mm;
        let max_sps = self.max_speed * self.step_per_mm;
        let accel = self.accelerate * self.step_per_mm;

        self.stepper
            .accel_move(steps, min_sps, max_sps, accel)
            .await;
        self.position += diff;
        self.position
    }

    pub async fn r#move(&mut self, length: i32) -> i32 {
        self.goto(self.position + length).await
    }

    pub fn step_per_mm(&self) -> u32 {
        self.step_per_mm
    }

    pub fn set_step_per_mm(&mut self, step_per_mm: u32) {
        self.step_per_mm = step_per_mm;
    }
}
