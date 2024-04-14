use std::path::{Path, PathBuf};
use std::sync::Arc;

use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream as _;
use v4l::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Orientation {
    Normal,
    Left,
    Right,
    Flip,
}
#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct CameraConfig {
    pub video_path: PathBuf,
    pub video_orientation: Orientation,
    //pub vertical_angle: f32,
    //pub horizontal_angle: f32,
    //pub distance: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            video_path: "/dev/video0".into(),
            video_orientation: Orientation::Normal,
            //vertical_angle: 60.0,
            //horizontal_angle: 60.0,
            //distance: 250.0,
        }
    }
}

impl CameraConfig {
    pub async fn capture(&mut self) -> anyhow::Result<DynamicImage> {
        let device = Device::with_path(&self.video_path)?;
        log::info!("Camera capturing");
        let ret = async_std::task::spawn_blocking(move || {
            let mut s = UserptrStream::new(&device, Type::VideoCapture)?;
            for _ in 0..5 {
                s.next()?;
            }
            let img = s.next()?.0;
            let img = image::load_from_memory(img)?;
            Ok(img)
        })
        .await;
        log::info!("Camera capturing done");
        ret
    }
}

pub struct Camera {
    device: Option<Arc<Device>>,
    video_path: PathBuf,
}

impl From<&CameraConfig> for Camera {
    fn from(value: &CameraConfig) -> Self {
        Self::from(value.clone())
    }
}
impl From<CameraConfig> for Camera {
    fn from(value: CameraConfig) -> Self {
        Self {
            device: None,
            video_path: value.video_path,
        }
    }
}

impl Camera {
    //pub fn new(config: &CameraConfig) -> anyhow::Result<Self> {
    //    Ok(Self {
    //        device: Arc::new(Device::with_path(config.video_path())?),
    //    })
    //}
    pub async fn capture(&mut self) -> anyhow::Result<DynamicImage> {
        let device = loop {
            if let Some(pin) = self.device.take() {
                break pin;
            } else {
                self.device
                    .replace(Arc::new(Device::with_path(&self.video_path)?));
            };
        };

        let dev = device.clone();

        log::info!("Camera capturing");
        let ret = async_std::task::spawn_blocking(move || {
            let mut s = UserptrStream::new(dev.as_ref(), Type::VideoCapture)?;
            for _ in 0..5 {
                s.next()?;
            }
            let img = s.next()?.0;
            let img = image::load_from_memory(img)?;
            //let width = img.width();
            //let height = img.height();

            //let min = width.min(height);

            //let off_v = (width - min) / 2;
            //let off_h = (height - min) / 2;

            //Ok(img.crop_imm(off_h, off_v, min, min))
            //Ok(img.crop_imm(0, 0, 640, 640))
            Ok(img)
        })
        .await;
        self.device.replace(device);
        log::info!("Camera capturing done");
        ret
    }

    //pub async fn list_camera(&self) -> anyhow::Result<Vec<PathBuf>> {
    //    let mut all_dev = udev::Enumerator::new()?;
    //    all_dev.match_subsystem("video4linux")?;

    //    Ok(all_dev
    //        .scan_devices()?
    //        .filter_map(|d| d.devnode().map(|p| p.to_owned()))
    //        .collect())
    //}

    //pub async fn init_camera(&mut self, path: &Path) -> anyhow::Result<()> {
    //    let mut all_dev = udev::Enumerator::new()?;
    //    all_dev.match_subsystem("video4linux")?;

    //    let path = all_dev
    //        .scan_devices()?
    //        .filter_map(|dev| dev.devnode().map(|p| p.to_owned()))
    //        .filter(|node| *node == path)
    //        .next()
    //        .ok_or_else(|| anyhow::anyhow!("can not find name"))?;

    //    let dev = Device::with_path(path)?;
    //    let mut s = UserptrStream::new(&dev, Type::VideoCapture)?;
    //    for _ in 0..10 {
    //        s.next()?;
    //    }
    //    self.device = Arc::new(dev);
    //    Ok(())
    //}
}
