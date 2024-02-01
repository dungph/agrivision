use async_std::sync::Mutex;
use image::DynamicImage;
use url::Url;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream as _;
use v4l::prelude::*;

use crate::gateway::broadcast;

static CAPTURE_DEV: Mutex<Option<CaptureDevice>> = Mutex::new(None);

enum CaptureDevice {
    Ip { snapshot: Url, stream: Url },
    V4l(Device),
}

pub async fn start_capture() {
    match crate::settings::camera() {
        crate::settings::CameraSetting::IpCamera {
            snapshot_url,
            stream_url,
        } => {
            CAPTURE_DEV.lock().await.replace(CaptureDevice::Ip {
                snapshot: snapshot_url,
                stream: stream_url,
            });
        }
        crate::settings::CameraSetting::LocalCamera { camera_id } => match Device::new(camera_id) {
            Ok(dev) => {
                CAPTURE_DEV.lock().await.replace(CaptureDevice::V4l(dev));
            }
            Err(e) => {
                broadcast(crate::message::Message::Error(e.to_string()));
            }
        },
    }
}

pub async fn capture() -> anyhow::Result<DynamicImage> {
    match CAPTURE_DEV.lock().await.as_mut() {
        Some(CaptureDevice::Ip {
            snapshot,
            stream: _,
        }) => capture_from_img_url(snapshot).await,
        Some(CaptureDevice::V4l(dev)) => capture_from_v4l(dev).await,
        None => Err(anyhow::anyhow!("Capture devices not specified")),
    }
}

async fn capture_from_img_url(url: &Url) -> anyhow::Result<DynamicImage> {
    let url = url.to_owned();
    let body = async_std::task::spawn_blocking(move || ureq::get(url.as_str()).call()).await?;
    let mut vec = Vec::new();
    body.into_reader().read_to_end(&mut vec)?;
    log::info!("Image fetched");
    Ok(image::load_from_memory(&vec)?)
}

async fn capture_from_v4l(dev: &Device) -> anyhow::Result<DynamicImage> {
    let mut s = UserptrStream::new(dev, Type::VideoCapture)?;
    async_std::task::spawn_blocking(move || {
        let img = s.next().unwrap().0;
        Ok(image::load_from_memory(img)?)
    })
    .await
}
