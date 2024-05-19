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

#[derive(Getters, Serialize, Deserialize)]
pub struct CameraConfig {
    #[serde(skip_serializing, skip_deserializing)]
    video: Option<UserptrStream>,
    pub video_path: PathBuf,
    pub video_orientation: Orientation,
    //pub vertical_angle: f32,
    //pub horizontal_angle: f32,
    //pub distance: f32,
}

impl Clone for CameraConfig {
    fn clone(&self) -> Self {
        CameraConfig {
            video: None,
            video_path: self.video_path.clone(),
            video_orientation: self.video_orientation.clone(),
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            video: None,
            video_path: "/dev/video0".into(),
            video_orientation: Orientation::Normal,
            //vertical_angle: 60.0,
            //horizontal_angle: 60.0,
            //distance: 250.0,
        }
    }
}

impl CameraConfig {
    pub fn capture_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        let device = Device::with_path(&self.video_path)?;
        let mut s = UserptrStream::new(&device, Type::VideoCapture)?;
        for _ in 0..5 {
            s.next()?;
        }
        let img = s.next()?.0;
        Ok(img.to_owned())
    }
    pub async fn capture(&mut self) -> anyhow::Result<DynamicImage> {
        let device = Device::with_path(&self.video_path)?;
        log::info!("Camera capturing");

        //let raw = self.next()?;
        //let ret = Ok(image::load_from_memory(&raw)?);
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
