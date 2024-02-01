use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use url::Url;

pub static SETTINGS: Lazy<Mutex<Setting>> = Lazy::new(|| Mutex::new(Setting::default()));
//pub static SETTINGS: Mutex<Setting> = Mutex::new(Setting {
//    model: ModelSetting {},
//    config: ConfigSetting {},
//    camera: CameraSetting::LocalCamera { camera_id: 0 },
//    restapi: RestAPISetting { port: 8080 },
//    x_linear_stepper: StepperSetting {},
//    y_linear_stepper: StepperSetting {},
//    z_linear_stepper: StepperSetting {},
//});

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Setting {
    vision: VisionSetting,
    #[serde(skip_serializing, default = "PathBuf::new")]
    config: PathBuf,
    camera: CameraSetting,
    restapi: RestAPISetting,
    linear_actuators: LinearsSetting,
}

pub fn open(path: &Path) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::options().read(true).open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let mut ret = toml::from_str::<Setting>(&string).expect("Incorrect config file!");
    ret.config = path.to_owned();
    println!("{:#?}", ret);
    *SETTINGS.lock().unwrap() = ret;
    Ok(())
}

pub fn save() -> Result<(), std::io::Error> {
    let settings = SETTINGS.lock().unwrap().clone();

    let mut file = std::fs::File::options()
        .write(true)
        .create(true)
        .open(&settings.config)?;
    let out = toml::to_string(&settings).unwrap();
    file.write_all(out.as_bytes())?;
    Ok(())
}

pub fn model() -> VisionSetting {
    SETTINGS.lock().unwrap().vision.clone()
}

pub fn set_model(model: VisionSetting) {
    SETTINGS.lock().unwrap().vision = model;
    save().ok();
}

pub fn restapi() -> RestAPISetting {
    SETTINGS.lock().unwrap().restapi.clone()
}

pub fn set_restapi(restapi: RestAPISetting) {
    SETTINGS.lock().unwrap().restapi = restapi;
    save().ok();
}

pub fn config() -> PathBuf {
    SETTINGS.lock().unwrap().config.clone()
}

pub fn set_config(config: PathBuf) {
    SETTINGS.lock().unwrap().config = config;
    save().ok();
}

pub fn camera() -> CameraSetting {
    SETTINGS.lock().unwrap().camera.clone()
}

pub fn set_camera(camera: CameraSetting) {
    SETTINGS.lock().unwrap().camera = camera;
    save().ok();
}

pub fn linear_actuators() -> LinearsSetting {
    SETTINGS.lock().unwrap().linear_actuators.clone()
}

pub fn set_linear_actuators(stepper: LinearsSetting) {
    SETTINGS.lock().unwrap().linear_actuators = stepper;
    save().ok();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSetting {
    pub path: PathBuf,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RestAPISetting {
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CameraSetting {
    IpCamera { snapshot_url: Url, stream_url: Url },
    LocalCamera { camera_id: usize },
}

impl Default for CameraSetting {
    fn default() -> Self {
        Self::LocalCamera { camera_id: 0 }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpCameraSetting {
    pub snapshot_url: Url,
    pub stream_url: Url,
}

impl Default for IpCameraSetting {
    fn default() -> Self {
        Self {
            snapshot_url: Url::parse("http://example.com/snapshot").unwrap(),
            stream_url: Url::parse("http://example.com/stream").unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LocalCameraSetting {
    pub camera_id: usize,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LinearsSetting {
    pub en_pin: usize,
    pub diag_pin: usize,
    pub x: StepperSetting,
    pub y: StepperSetting,
    pub z: StepperSetting,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StepperSetting {
    pub reversed: bool,
    pub step_pin: usize,
    pub dir_pin: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VisionSetting {
    pub path: PathBuf,
    pub size: ModelSize,
    pub num_classes: usize,
    pub nms_threshold: f32,
    pub acc_threshold: f32,
    pub interval: u64,
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
