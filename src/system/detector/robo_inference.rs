use std::io::Cursor;

use base64::{prelude::BASE64_STANDARD, Engine};
use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use surf::Body;

#[derive(Clone, Debug, Getters, Serialize, Deserialize, Default)]
pub struct RoboConfig {
    project_name: String,
    version: usize,
    api_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct Query<'a> {
    api_key: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    predictions: Vec<RawDetectionResult>,
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct RawDetectionResult {
    pub class: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RoboConfig {
    pub async fn detect(
        &mut self,
        img: &DynamicImage,
    ) -> anyhow::Result<Vec<super::DetectionResult>> {
        let mut buf = Vec::new();
        let mut cur = Cursor::new(&mut buf);
        img.write_to(&mut cur, image::ImageFormat::Jpeg).unwrap();

        let ret: QueryResult = surf::post(format!(
            "https://detect.roboflow.com/{}/{}?api_key={}",
            self.project_name, self.version, self.api_key,
        ))
        .content_type("application/x-www-form-urlencoded")
        .body(Body::from_bytes(
            BASE64_STANDARD.encode(buf).as_bytes().to_owned(),
        ))
        .await
        .unwrap()
        .body_json()
        .await
        .unwrap();
        Ok(ret
            .predictions
            .into_iter()
            .map(|r| super::DetectionResult {
                class: r.class.parse().unwrap(),
                x: r.x as u32,
                y: r.y as u32,
                width: r.width as u32,
                height: r.height as u32,
            })
            .collect())
    }
}
