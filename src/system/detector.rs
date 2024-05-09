use derive_getters::Getters;
use serde::{Deserialize, Serialize};

mod local_yolov8;
mod robo_inference;
mod yolov8_algorithm;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DetectorConfig {
    YoloV8(local_yolov8::Yolov8Config),
    Robo(robo_inference::RoboConfig),
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct DetectionResult {
    pub class: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl DetectionResult {
    fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
    fn distance_from_point(&self, point: (u32, u32)) -> u32 {
        let center = self.center();
        point.0.abs_diff(center.0) * point.0.abs_diff(center.0)
            + point.1.abs_diff(center.1) * point.1.abs_diff(center.1)
    }
    fn point_in_box(&self, point: (u32, u32)) -> bool {
        self.x <= point.0
            && point.0 <= self.x + self.width
            && self.y <= point.1
            && point.1 <= self.y + self.height
    }
}

impl Default for DetectorConfig {
    fn default() -> Self {
        DetectorConfig::YoloV8(local_yolov8::Yolov8Config::default())
    }
}

impl DetectorConfig {
    pub async fn detect(&mut self, img: &image::DynamicImage) -> anyhow::Result<DetectionResult> {
        let detections = match self {
            DetectorConfig::YoloV8(config) => config.get_bounding_boxes(img).await?,
            DetectorConfig::Robo(config) => config.detect(img).await?,
        };

        let cx = img.width() / 2;
        let cy = img.height() / 2;

        let neer_center = |this: DetectionResult, other: DetectionResult| -> DetectionResult {
            if this.distance_from_point((cx, cy)) > other.distance_from_point((cx, cy)) {
                other
            } else {
                this
            }
        };

        let ret = detections
            .into_iter()
            .filter(|d| d.point_in_box((cx, cy)))
            .reduce(neer_center)
            .unwrap_or_else(|| DetectionResult {
                class: "unknown".to_owned(),
                x: cx,
                y: cy,
                width: 1,
                height: 1,
            });
        Ok(ret)
    }
}
