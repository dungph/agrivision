use std::path::Path;

use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Stage {
    Young,
    Ready,
    Old,
    Empty,
    Unknown,
}

impl std::str::FromStr for Stage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "young" => Self::Young,
            "ready" => Self::Young,
            "old" => Self::Young,
            "empty" => Self::Young,
            "unknown" => Self::Young,
            s => return Err(anyhow::anyhow!("Cannot parse {s}")),
        })
    }
}

impl ToString for Stage {
    fn to_string(&self) -> String {
        match self {
            Stage::Young => "Young",
            Stage::Ready => "Ready",
            Stage::Old => "Old",
            Stage::Empty => "Empty",
            Stage::Unknown => "Unknown",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Getters)]
pub struct DetectResult {
    pub x: u32,
    pub y: u32,
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
    pub stage: Stage,
    pub image: DynamicImage,
    pub timestamp: u64,
}

impl DetectResult {
    pub fn parse_name(file_name: &str) -> anyhow::Result<Option<Self>> {
        let fields = file_name.split('-').collect::<Vec<&str>>();
        if fields.len() == 9 {
            let ret = DetectResult {
                stage: fields[1].parse()?,
                x: fields[2].parse()?,
                y: fields[3].parse()?,
                top: fields[4].parse()?,
                left: fields[5].parse()?,
                right: fields[6].parse()?,
                bottom: fields[7].parse()?,
                timestamp: fields[8].parse()?,
                image: DynamicImage::new_rgb8(10, 10),
            };
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }
    pub fn open(dir: &Path, file_name: &str) -> anyhow::Result<Option<Self>> {
        let path = dir.join(file_name);
        let image = image::open(path)?;
        if let Some(mut res) = Self::parse_name(file_name)? {
            res.image = image;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    pub fn list_pot(dir: &Path) -> anyhow::Result<Vec<String>> {
        Ok(std::fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|entry| entry.is_file())
            .filter(|path| path.file_name().is_some())
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect())
    }
    pub fn save(&self, dir: &Path) -> anyhow::Result<()> {
        let path = dir.join(format!(
            "Detected-{}-{}-{}-{}-{}-{}-{}-{}.jpg",
            self.stage.to_string(),
            self.x,
            self.y,
            self.top,
            self.left,
            self.right,
            self.bottom,
            self.timestamp
        ));
        self.image.save(path)?;
        Ok(())
    }
}
