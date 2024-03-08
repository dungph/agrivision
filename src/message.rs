use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pot {
    pub x: u32,
    pub y: u32,
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
    pub stage: State,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Young,
    Ready,
    Old,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    GetListPot,
    ReportPot(Pot),

    AutoWater(bool),
    ManualWater { x: u32, y: u32 },

    AutoCheck(bool),
    Check { x: u32, y: u32 },

    Status(String),
    Error(String),
}
