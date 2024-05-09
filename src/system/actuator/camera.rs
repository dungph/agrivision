use std::path::PathBuf;

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
