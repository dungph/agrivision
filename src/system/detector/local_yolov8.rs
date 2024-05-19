use std::path::PathBuf;
use std::sync::Arc;

use async_std::task::spawn_blocking;
use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::system::detector::yolov8_algorithm::Multiples;

use super::yolov8_algorithm::YoloV8;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ModelSize {
    #[default]
    N,
    S,
    M,
    L,
    X,
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Yolov8Config {
    model_path: PathBuf,
    model_size: ModelSize,
    num_classes: usize,
    nms_threshold: f32,
    acc_threshold: f32,
}

impl Yolov8Config {
    pub async fn get_bounding_boxes(
        &self,
        img: &DynamicImage,
    ) -> anyhow::Result<Vec<super::DetectionResult>> {
        let multiples = match self.model_size() {
            ModelSize::M => Multiples::m(),
            ModelSize::S => Multiples::s(),
            ModelSize::X => Multiples::x(),
            ModelSize::N => Multiples::n(),
            ModelSize::L => Multiples::l(),
        };

        let yolo = Arc::new(YoloV8::from_path_safetensors(
            self.model_path(),
            multiples,
            *self.num_classes(),
        )?);

        let acc = self.acc_threshold;
        let nms = self.nms_threshold;
        let img = img.clone();
        log::info!("Model running");
        let ret = spawn_blocking(move || yolo.report_boxes(&img, acc, nms)).await?;
        log::info!("Model running done");
        Ok(ret
            .into_iter()
            .map(|ret| super::DetectionResult {
                class: obj_id_to_class(ret.object_id),
                x: ret.x,
                y: ret.y,
                width: ret.w,
                height: ret.h,
            })
            .collect())
    }
}

impl Default for Yolov8Config {
    fn default() -> Self {
        Yolov8Config {
            model_path: "./best.safetensors".into(),
            model_size: ModelSize::N,
            num_classes: 3,
            nms_threshold: 0.5,
            acc_threshold: 0.5,
        }
    }
}

fn obj_id_to_class(id: usize) -> String {
    match id {
        0 => "young",
        1 => "ready",
        2 => "old",
        3 => "empty",
        _ => "unknown",
    }
    .to_owned()
}
