use std::path::Path;
use std::sync::Arc;

use image::DynamicImage;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream as _;
use v4l::prelude::*;

pub struct Camera {
    device: Arc<Device>,
}
impl Camera {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            device: Arc::new(Device::with_path(path)?),
        })
    }

    pub async fn capture(&self) -> anyhow::Result<DynamicImage> {
        let dev = self.device.clone();
        async_std::task::spawn_blocking(move || {
            let mut s = UserptrStream::new(dev.as_ref(), Type::VideoCapture)?;
            let img = s.next().unwrap().0;
            Ok(image::load_from_memory(img)?)
        })
        .await
    }
}
