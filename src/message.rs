use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Young,
    Ready,
    Old,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    GetReport,
    GetListPot,
    GetMovingState,
    AutoWater(bool),
    ManualWater {
        x: u32,
        y: u32,
    },
    GetWateringState,
    AutoCheck(bool),
    ManualCheck {
        x: u32,
        y: u32,
    },
    GetCapturingState,
    ReportPot {
        x: u32,
        y: u32,
    },
    ReportMoving(bool),

    GetAutoWater,
    ReportAutoWater(bool),
    ReportWater {
        x: u32,
        y: u32,
        timestamp: u64,
    },
    ReportWatering(bool),

    GetAutoCheck,
    ReportAutoCheck(bool),
    ReportCheck {
        x: u32,
        y: u32,
        top: u32,
        left: u32,
        bottom: u32,
        right: u32,
        stage: Stage,
        timestamp: u64,
    },
    ReportCapturing(bool),
    ReportImageFile(String),
    ReportPotImageFile {
        x: u32,
        y: u32,
        file_path: String,
    },

    Status(String),
    Error(String),
}
