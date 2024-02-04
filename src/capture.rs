use async_std::sync::Mutex;
use image::DynamicImage;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream as _;
use v4l::prelude::*;

use crate::gateway::broadcast;

static CAPTURE_DEV: Mutex<Option<Device>> = Mutex::new(None);

pub async fn start_capture() {
    let camera = crate::settings::camera();
    match Device::with_path(camera.video_path) {
        Ok(dev) => {
            CAPTURE_DEV.lock().await.replace(dev);
        }
        Err(e) => {
            broadcast(crate::message::Message::Error(e.to_string()));
        }
    }
}

pub async fn capture() -> anyhow::Result<DynamicImage> {
    match CAPTURE_DEV.lock().await.as_mut() {
        Some(dev) => capture_from_v4l(dev).await,
        None => Err(anyhow::anyhow!("Capture devices not specified")),
    }
}

async fn capture_from_v4l(dev: &Device) -> anyhow::Result<DynamicImage> {
    let mut s = UserptrStream::new(dev, Type::VideoCapture)?;
    async_std::task::spawn_blocking(move || {
        let img = s.next().unwrap().0;
        Ok(image::load_from_memory(img)?)
    })
    .await
}
