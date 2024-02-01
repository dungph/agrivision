use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectionBox {
    pub object_id: usize,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Hello,

    ReportPosition(i32, i32, i32),
    ReportListBox(Vec<DetectionBox>),

    ControlGoto(i32, i32, i32),
    ControlMove(i32, i32, i32),
    ControlFanState(bool),
    ControlPumpState(bool),

    Status(String),
    Error(String),
}
