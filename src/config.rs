use std::{
    io::Read,
    path::{Path, PathBuf},
};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    http: Http,
    positions: Positions,
    camera: Camera,
    yolo: Yolo,
    watering: Water,
    linear_x: Linear,
    linear_y: Linear,
}

impl Config {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let mut file = std::fs::File::options().read(true).open(path)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Ok(toml::from_str::<Config>(&string)?)
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Positions {
    positions: Vec<[u32; 2]>,
}

impl Default for Positions {
    fn default() -> Self {
        Self {
            positions: vec![[0, 0], [1, 1]],
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Http {
    listen_socket: std::net::SocketAddr,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            listen_socket: "0.0.0.0:8080".parse().unwrap(),
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Water {
    pin: GpioPin,
    interval: u32,
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Camera {
    video_path: PathBuf,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            video_path: "/dev/video0".into(),
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Linear {
    reverse: bool,
    en_pin: GpioPin,
    step_pin: GpioPin,
    dir_pin: GpioPin,
    min_mm_per_s: u32,
    max_mm_per_s: u32,
    accelerate: u32,
    step_per_ms: u32,
}

impl Default for Linear {
    fn default() -> Self {
        Self {
            reverse: false,
            en_pin: Default::default(),
            step_pin: Default::default(),
            dir_pin: Default::default(),
            min_mm_per_s: 5,
            max_mm_per_s: 200,
            accelerate: 100,
            step_per_ms: 40,
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct GpioPin {
    chip: PathBuf,
    line: u32,
}

impl Default for GpioPin {
    fn default() -> Self {
        Self {
            chip: "/dev/gpiochip0".into(),
            line: 0,
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Yolo {
    model_path: PathBuf,
    model_size: ModelSize,
    num_classes: usize,
    nms_threshold: f32,
    acc_threshold: f32,
}

impl Default for Yolo {
    fn default() -> Self {
        Yolo {
            model_path: "./best.safetensors".into(),
            model_size: ModelSize::N,
            num_classes: 3,
            nms_threshold: 0.5,
            acc_threshold: 0.5,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ModelSize {
    #[default]
    N,
    S,
    M,
    L,
    X,
}
