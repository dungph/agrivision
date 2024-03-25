use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Young,
    Ready,
    Old,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InMsg {
    GetReport,
    GetListPot,

    GetMovingState,
    GetWateringState,
    GetCapturingState,

    GetAutoWater,
    GetAutoCheck,

    SetAutoWater(bool),
    SetAutoCheck(bool),

    Water { x: u32, y: u32 },
    Check { x: u32, y: u32 },
    Shutdown,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug)]
pub enum OutMsg<Image = ()> {
    ReportMoving(bool),
    ReportWatering(bool),
    ReportCapturing(bool),
    ReportAutoWater(bool),
    ReportAutoCheck(bool),

    ReportPot {
        x: u32,
        y: u32,
    },
    ReportWater {
        x: u32,
        y: u32,
        timestamp: u64,
    },
    ReportCheck {
        x: u32,
        y: u32,
        top: u32,
        left: u32,
        bottom: u32,
        right: u32,
        stage: Stage,
        timestamp: u64,
        #[derivative(Debug = "ignore")]
        image: Image,
    },
    ReportImage(Image),
}
