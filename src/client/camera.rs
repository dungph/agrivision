use std::time::Duration;

use async_std::task::spawn;
use serde::Deserialize;
use tide::{Request, Response};

use crate::{database, system};

use super::get_user;

pub async fn snapshot(req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin || user.is_manager || user.is_watcher => {
            let f = system::capture_raw().await?;
            let response = tide::Response::builder(200)
                .header("Access-Control-Allow-Origin", "*")
                .content_type("image/jpeg")
                .body(tide::Body::from_bytes(f))
                .build();
            Ok(response)
        }
        _ => Ok(Response::new(403)),
    }
}
pub async fn stream(req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin || user.is_manager || user.is_watcher => {
            const BOUNDARY: &str = "mjpeg-boundary";
            // Prepare the response with the correct header
            let (writer, drain) = async_std::channel::bounded(2);
            let buf_drain = futures::stream::TryStreamExt::into_async_read(drain);

            // Prepare the response with the correct header
            let response = tide::Response::builder(200)
                .header("Access-Control-Allow-Origin", "*")
                .content_type(format!("multipart/x-mixed-replace;boundary={BOUNDARY}").as_str())
                .body(tide::Body::from_reader(buf_drain, None))
                .build();

            spawn(async move {
                loop {
                    async_std::task::sleep(Duration::from_millis(200)).await;
                    let f = system::capture_raw().await.unwrap();

                    // Start the buffer that we'll send using the boundary and some multi-part http header
                    // context.
                    let buffer = format!(
                        "--{BOUNDARY}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        f.len(),
                    )
                    .into_bytes();

                    if writer.try_send(Ok(buffer)).is_ok() {
                    } else {
                        break;
                    }

                    if writer.try_send(Ok(f.to_owned())).is_ok() {
                    } else {
                        break;
                    }
                }
            });
            Ok(response)
        }
        _ => Ok(Response::new(403)),
    }
}

pub async fn image(req: Request<()>) -> tide::Result {
    match get_user(&req).await? {
        Some(user) if user.is_admin || user.is_manager || user.is_watcher => {
            #[derive(Deserialize)]
            struct Query {
                id: u64,
            }
            if let Ok(Query { id }) = req.query() {
                let image = database::get_image(id as i64).await?;
                let response = tide::Response::builder(200)
                    .header("Access-Control-Allow-Origin", "*")
                    .content_type("image/jpeg")
                    .body(tide::Body::from_bytes(image))
                    .build();
                Ok(response)
            } else {
                Ok(Response::new(400))
            }
        }
        _ => Ok(Response::new(403)),
    }
}
