use std::sync::Arc;
use std::{collections::BTreeMap, path::PathBuf};

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
    class_mapping: BTreeMap<String, String>,
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
            self.class_mapping.len(),
        )?);

        let acc = self.acc_threshold;
        let nms = self.nms_threshold;
        let img = img.clone();
        log::info!("Model running");
        let ret = spawn_blocking(move || yolo.report_boxes(&img, acc, nms)).await?;
        log::info!("Model running done");
        Ok(ret
            .into_iter()
            .map(|ret| {
                dbg!(ret.object_id);
                super::DetectionResult {
                    class: self
                        .class_mapping
                        .get(&ret.object_id.to_string())
                        .map(|s| s.to_owned())
                        .unwrap_or("unknown".to_owned()),
                    x: ret.x,
                    y: ret.y,
                    width: ret.w,
                    height: ret.h,
                }
            })
            .collect())
    }
}

impl Default for Yolov8Config {
    fn default() -> Self {
        let mut cmap = BTreeMap::new();
        cmap.insert("0".to_owned(), "empty".to_owned());
        cmap.insert("1".to_owned(), "young".to_owned());
        cmap.insert("2".to_owned(), "ready".to_owned());
        cmap.insert("3".to_owned(), "old".to_owned());
        Yolov8Config {
            model_path: "./best.safetensors".into(),
            model_size: ModelSize::N,
            class_mapping: cmap,
            nms_threshold: 0.3,
            acc_threshold: 0.5,
        }
    }
}
