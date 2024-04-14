use std::{collections::VecDeque, io::Cursor};

use base64::Engine;
use image::ImageFormat;
use serde::{Deserialize, Serialize};

use crate::system::{CaptureResult, DetectResult, Stage, System, WaterResult};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IncomingMessage {
    GetListPot,

    GetAutoWater,
    GetAutoCheck,

    SetAutoWater(bool),
    SetAutoCheck(bool),

    Goto { x: u32, y: u32 },
    SetPot { x: u32, y: u32 },
    RemovePot { x: u32, y: u32 },

    Water { x: u32, y: u32 },
    Check { x: u32, y: u32 },

    GetAllWater { x: u32, y: u32 },
    GetAllCheck { x: u32, y: u32 },

    GetLastWater { x: u32, y: u32 },
    GetLastCheck { x: u32, y: u32 },

    Shutdown,
}

impl std::fmt::Display for IncomingMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum OutgoingMessage {
    AutoWater {
        state: bool,
    },

    AutoCheck {
        state: bool,
    },

    Pot {
        x: u32,
        y: u32,
    },
    Water {
        x: u32,
        y: u32,
        timestamp: u64,
    },
    Check {
        x: u32,
        y: u32,
        top: u32,
        left: u32,
        bottom: u32,
        right: u32,
        stage: Stage,
        image: String,
        timestamp: u64,
    },
    Capture {
        x: u32,
        y: u32,
        image: String,
        timestamp: u64,
    },
}

impl std::fmt::Display for OutgoingMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut this = self.clone();
        match &mut this {
            Self::Check {
                x,
                y,
                top,
                left,
                bottom,
                right,
                stage,
                image,
                timestamp,
            } => {
                *image = String::new();
            }
            Self::Capture {
                x,
                y,
                image,
                timestamp,
            } => {
                *image = String::new();
            }
            _ => (),
        }
        write!(f, "{:}", this)
    }
}

impl From<CaptureResult> for OutgoingMessage {
    fn from(value: CaptureResult) -> Self {
        let mut img = Vec::new();
        value
            .image()
            .write_to(&mut Cursor::new(&mut img), ImageFormat::Jpeg)
            .ok();
        Self::Capture {
            x: *value.x(),
            y: *value.y(),
            image: base64::prelude::BASE64_STANDARD.encode(img),
            timestamp: *value.timestamp(),
        }
    }
}
impl From<DetectResult> for OutgoingMessage {
    fn from(value: DetectResult) -> Self {
        let mut img = Vec::new();
        value
            .image()
            .write_to(&mut Cursor::new(&mut img), ImageFormat::Jpeg)
            .ok();
        Self::Check {
            x: *value.x(),
            y: *value.y(),
            top: *value.top(),
            left: *value.left(),
            right: *value.right(),
            bottom: *value.bottom(),
            image: base64::prelude::BASE64_STANDARD.encode(img),
            stage: value.stage().clone(),
            timestamp: *value.timestamp(),
        }
    }
}
impl From<WaterResult> for OutgoingMessage {
    fn from(value: WaterResult) -> Self {
        Self::Water {
            x: *value.x(),
            y: *value.y(),
            timestamp: *value.timestamp(),
        }
    }
}
