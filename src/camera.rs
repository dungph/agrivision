use std::path::Path;
use std::sync::Arc;

use image::DynamicImage;
use imageproc::rect::Rect;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream as _;
use v4l::prelude::*;

pub trait CameraIf {
    fn capture(&self) -> impl std::future::Future<Output = anyhow::Result<DynamicImage>>;
}

impl CameraIf for Camera {
    async fn capture(&self) -> anyhow::Result<DynamicImage> {
        self.capture().await
    }
}
impl CameraIf for DummyCamera {
    async fn capture(&self) -> anyhow::Result<DynamicImage> {
        let mut img = image::DynamicImage::new_rgb8(1280, 720);
        imageproc::drawing::draw_filled_rect_mut(
            &mut img,
            Rect::at(
                (rand::random::<u32>() % 400) as i32,
                (rand::random::<u32>() % 400) as i32,
            )
            .of_size(
                rand::random::<u32>() % 200 + 50,
                rand::random::<u32>() % 200 + 50,
            ),
            image::Rgba([255, 0, 0, 100]),
        );
        Ok(img)
    }
}
pub struct DummyCamera;

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
