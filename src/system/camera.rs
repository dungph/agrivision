use std::path::PathBuf;

use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream as _;
use v4l::video::Capture;
use v4l::{prelude::*, Format, FourCC};

#[derive(Getters, Serialize, Deserialize)]
pub struct CameraConfig {
    #[serde(skip_serializing, skip_deserializing)]
    pub video: Option<UserptrStream>,
    pub video_path: PathBuf,
}

impl Clone for CameraConfig {
    fn clone(&self) -> Self {
        CameraConfig {
            video: None,
            video_path: self.video_path.clone(),
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            video: None,
            video_path: "/dev/video0".into(),
        }
    }
}

impl CameraConfig {
    pub async fn capture_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        if self.video.is_none() {
            let dev = Device::with_path(&self.video_path)?;
            dev.set_format(&Format::new(1280, 720, FourCC::new(b"MJPG")))?;
            let stream = UserptrStream::with_buffers(&dev, Type::VideoCapture, 1)?;
            self.video.replace(stream);
        }

        let img = self.video.as_mut().unwrap().next()?.0.to_owned();
        Ok(img)
    }
    pub async fn capture(&mut self) -> anyhow::Result<DynamicImage> {
        log::info!("Camera capturing");
        let img = self.capture_raw().await?;
        let img = image::load_from_memory(&img)?;
        log::info!("Camera capturing done");
        Ok(img)
    }
}
