mod actuator;
mod camera;
mod detector;
mod gpio_pin;
mod linear;
mod watering;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_std::channel::{bounded, Receiver};
use async_std::sync::Mutex;
use async_std::task::sleep;
use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use self::actuator::ActuatorProfile;
pub use self::detection_result::{DetectResult, Stage};
use self::detector::{Yolov8, Yolov8Config};

mod detection_result;

#[derive(Clone, Debug, Getters)]
pub struct WaterResult {
    pub x: u32,
    pub y: u32,
    pub timestamp: u64,
    pub image: DynamicImage,
}

#[derive(Clone, Debug, Getters)]
pub struct CaptureResult {
    pub x: u32,
    pub y: u32,
    pub image: DynamicImage,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize, Default)]
pub struct GeneralProfile {
    data_dir: PathBuf,

    auto_check: bool,

    auto_water: bool,
    auto_check_period: u64,
    auto_water_period: u64,

    auto_water_empty: bool,
    auto_water_young: bool,
    auto_water_ready: bool,
    auto_water_old: bool,
    auto_water_unknown: bool,

    water_empty_duration: u32,
    water_young_duration: u32,
    water_ready_duration: u32,
    water_old_duration: u32,
    water_unknown_duration: u32,
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize, Default)]
pub struct LocalSystemConfig {
    general: GeneralProfile,
    actuators: ActuatorProfile,
    detector: Yolov8Config,
    positions: BTreeSet<(u32, u32)>,
}
#[derive(Clone)]
pub struct System {
    general: Arc<Mutex<GeneralProfile>>,
    actuator: Arc<Mutex<ActuatorProfile>>,
    detector: Arc<Mutex<Yolov8Config>>,
    positions: Arc<Mutex<BTreeSet<(u32, u32)>>>,
    config_path: PathBuf,
}

impl System {
    pub fn from_path(config_path: &Path) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(config_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let config: LocalSystemConfig = toml::from_str(&data)?;
        Ok(Self {
            general: Arc::new(Mutex::new(config.general)),
            actuator: Arc::new(Mutex::new(config.actuators)),
            detector: Arc::new(Mutex::new(config.detector)),
            positions: Arc::new(Mutex::new(config.positions)),
            config_path: config_path.to_owned(),
        })
    }

    pub async fn sync_config(&self) -> anyhow::Result<()> {
        let general = self.general.lock().await.clone();
        let actuators = self.actuator.lock().await.clone();
        let detector = self.detector.lock().await.clone();
        let positions = self.positions.lock().await.clone();
        let config = LocalSystemConfig {
            general,
            actuators,
            detector,
            positions,
        };

        let conf_data = toml::to_string(&config)?;

        let mut file = std::fs::File::options()
            .write(true)
            .truncate(true)
            .open(&self.config_path)?;
        file.write_all(conf_data.as_bytes())?;
        Ok(())
    }

    pub async fn poweroff(&self) -> anyhow::Result<()> {
        self.actuator.lock().await.goto2(0, 0).await?;
        Ok(())
    }

    pub async fn positions(&self) -> anyhow::Result<BTreeSet<(u32, u32)>> {
        Ok(self.positions.lock().await.clone())
    }
    pub async fn add_positions(&self, x: u32, y: u32) -> anyhow::Result<()> {
        self.positions.lock().await.insert((x, y));
        self.sync_config().await?;
        Ok(())
    }
    pub async fn remove_positions(&self, x: u32, y: u32) -> anyhow::Result<()> {
        self.positions.lock().await.remove(&(x, y));
        self.sync_config().await?;
        Ok(())
    }

    pub async fn get_last_check_report(
        &self,
        x: u32,
        y: u32,
    ) -> anyhow::Result<Option<DetectResult>> {
        let dir = &self.general.lock().await.data_dir;
        let list = DetectResult::list_pot(dir)?;

        let ret = list
            .into_iter()
            .filter_map(|name| {
                Some((
                    name.clone(),
                    DetectResult::parse_name(&name).ok().flatten()?,
                ))
            })
            .filter(|(_name, res)| res.x == x && res.y == y)
            .reduce(|acc, res| {
                if res.1.timestamp > acc.1.timestamp {
                    res
                } else {
                    acc
                }
            });
        if let Some(ret) = ret {
            Ok(DetectResult::open(dir.as_path(), ret.0.as_ref())?)
        } else {
            Ok(None)
        }
    }
    pub async fn get_all_check_report(
        &self,
        _x: u32,
        _y: u32,
    ) -> anyhow::Result<BTreeMap<(u32, u32), DetectResult>> {
        Ok(BTreeMap::new())
    }

    pub async fn get_last_water_report(
        &self,
        x: u32,
        y: u32,
    ) -> anyhow::Result<Option<WaterResult>> {
        Ok(None)
    }

    pub async fn get_all_water_report(
        &self,
        _x: u32,
        _y: u32,
    ) -> anyhow::Result<BTreeMap<(u32, u32), WaterResult>> {
        Ok(BTreeMap::new())
    }

    pub async fn auto_water(&self) -> anyhow::Result<bool> {
        Ok(self.general.lock().await.auto_water)
    }

    pub async fn auto_check(&self) -> anyhow::Result<bool> {
        Ok(self.general.lock().await.auto_check)
    }
    pub async fn set_auto_water(&self, state: bool) -> anyhow::Result<()> {
        self.general.lock().await.auto_water = state;
        self.sync_config().await?;
        Ok(())
    }
    pub async fn set_auto_check(&self, state: bool) -> anyhow::Result<()> {
        self.general.lock().await.auto_check = state;
        self.sync_config().await?;
        Ok(())
    }

    pub async fn water_at(&self, x: u32, y: u32) -> anyhow::Result<WaterResult> {
        let img = self
            .actuator
            .lock()
            .await
            .water_at(x, y, Duration::from_secs(2))
            .await?;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let ret = WaterResult {
            x,
            y,
            image: img,
            timestamp: time,
        };
        Ok(ret)
    }

    pub async fn capture_at(&self, x: u32, y: u32) -> anyhow::Result<CaptureResult> {
        let image = self.actuator.lock().await.capture_at(x, y).await?;
        let res = CaptureResult {
            x,
            y,
            image,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        Ok(res)
    }

    pub async fn check_at(&self, x: u32, y: u32) -> anyhow::Result<DetectResult> {
        let CaptureResult {
            x,
            y,
            mut image,
            timestamp,
        } = self.capture_at(x, y).await?;

        let detections = self.detector.lock().await.get_bounding_box(&image).await?;

        let img_w = image.width();
        let img_h = image.height();

        for bbox in detections.iter() {
            imageproc::drawing::draw_hollow_rect_mut(
                &mut image,
                imageproc::rect::Rect::at(*bbox.x() as i32, *bbox.y() as i32)
                    .of_size(*bbox.w(), *bbox.h()),
                image::Rgba([255, 0, 0, 100]),
            )
        }

        let detection = detections
            .iter()
            .filter(|d| *d.x() < img_w / 2 && img_w / 2 < d.x() + d.w())
            .find(|d| *d.y() < img_h / 2 && img_h / 2 < d.y() + d.h());

        let ret = if let Some(detection) = detection {
            let stage = match detection.object_id() {
                0 => Stage::Young,
                1 => Stage::Ready,
                2 => Stage::Old,
                3 => Stage::Empty,
                4 => Stage::Unknown,
                _ => Stage::Unknown,
            };

            DetectResult {
                x,
                y,
                top: 20,
                left: 20,
                right: 20,
                bottom: 20,
                image,
                stage,
                timestamp,
            }
        } else {
            DetectResult {
                x,
                y,
                top: 20,
                left: 20,
                right: 20,
                bottom: 20,
                image,
                stage: Stage::Unknown,
                timestamp,
            }
        };
        ret.save(&self.general.lock().await.data_dir)?;
        Ok(ret)
    }

    pub async fn start_automation<T>(&self) -> anyhow::Result<Receiver<T>>
    where
        T: From<WaterResult> + From<DetectResult> + Send + Sync + 'static,
    {
        let (tx, rx) = bounded(100);

        let this = self.clone();
        async_std::task::spawn(async move {
            loop {
                if false {
                    break;
                }

                let list_water: BTreeMap<(u32, u32), SystemTime>;
                let mut last_check = UNIX_EPOCH;

                let check_period = this.general.lock().await.auto_water_period;

                let elapsed = SystemTime::now()
                    .duration_since(last_check)
                    .unwrap()
                    .as_secs();

                if elapsed <= check_period {
                    for (x, y) in this.positions().await? {
                        let exist = this
                            .positions()
                            .await?
                            .iter()
                            .any(|(px, py)| *px == x && *py == y);

                        if exist && this.auto_water().await? {
                            let res = this.check_at(x, y).await?;
                            tx.send(res.into()).await?;
                        }
                    }
                    last_check = SystemTime::now();
                }
                sleep(Duration::from_secs(1)).await;
            }
            Ok(()) as anyhow::Result<()>
        });
        Ok(rx)
    }
}
//#[derive(Clone, Debug, Getters, Serialize, Deserialize, Default)]
//pub struct LocalSystemConfig {
//    data_dir: PathBuf,
//    actuators: actuator::LocalActuatorConfig,
//    detector: detector::Yolov8Config,
//
//    auto_check: bool,
//
//    auto_water: bool,
//    auto_check_period: u64,
//    auto_water_period: u64,
//
//    auto_water_empty: bool,
//    auto_water_young: bool,
//    auto_water_ready: bool,
//    auto_water_old: bool,
//    auto_water_unknown: bool,
//
//    water_empty_duration: u32,
//    water_young_duration: u32,
//    water_ready_duration: u32,
//    water_old_duration: u32,
//    water_unknown_duration: u32,
//
//    positions: BTreeSet<(u32, u32)>,
//}
//
//#[derive(Clone)]
//pub struct System {
//    inner: Arc<Mutex<Actuators>>,
//    detector: Arc<Yolov8>,
//    config: Arc<Mutex<LocalSystemConfig>>,
//    config_path: PathBuf,
//}
//
//impl System {
//    pub fn from_path(config_path: &Path) -> anyhow::Result<Self> {
//        let mut file = std::fs::File::open(config_path)?;
//        let mut data = String::new();
//        file.read_to_string(&mut data)?;
//        let config: LocalSystemConfig = toml::from_str(&data)?;
//        Ok(Self {
//            inner: Arc::new(Mutex::new(Actuators::new(config.actuators())?)),
//            detector: Arc::new(Yolov8::new(config.detector())?),
//            config: Arc::new(Mutex::new(config)),
//            config_path: config_path.to_owned(),
//        })
//    }
//
//    pub async fn sync_config(&self) -> anyhow::Result<()> {
//        let mut config = self.config.lock().await;
//        config.actuators = self.inner.lock().await.current_config()?;
//
//        let conf_data = toml::to_string(&*config)?;
//
//        let mut file = std::fs::File::options()
//            .write(true)
//            .truncate(true)
//            .open(&self.config_path)?;
//        file.write_all(conf_data.as_bytes())?;
//        Ok(())
//    }
//
//    pub async fn poweroff(&self) -> anyhow::Result<()> {
//        self.inner.lock().await.goto(0, 0).await?;
//        Ok(())
//    }
//
//    pub async fn positions(&self) -> anyhow::Result<BTreeSet<(u32, u32)>> {
//        Ok(self.config.lock().await.positions.clone())
//    }
//    pub async fn add_positions(&self, x: u32, y: u32) -> anyhow::Result<()> {
//        self.config.lock().await.positions.insert((x, y));
//        dbg!(self.sync_config().await)?;
//        Ok(())
//    }
//    pub async fn remove_positions(&self, x: u32, y: u32) -> anyhow::Result<()> {
//        self.config.lock().await.positions.remove(&(x, y));
//        self.sync_config().await?;
//        Ok(())
//    }
//
//    pub async fn get_last_check_report(
//        &self,
//        x: u32,
//        y: u32,
//    ) -> anyhow::Result<Option<DetectResult>> {
//        let dir = &self.config.lock().await.data_dir;
//        let list = DetectResult::list_pot(dir)?;
//
//        let ret = list
//            .into_iter()
//            .filter_map(|name| {
//                Some((
//                    name.clone(),
//                    DetectResult::parse_name(&name).ok().flatten()?,
//                ))
//            })
//            .filter(|(_name, res)| res.x == x && res.y == y)
//            .reduce(|acc, res| {
//                if res.1.timestamp > acc.1.timestamp {
//                    res
//                } else {
//                    acc
//                }
//            });
//        if let Some(ret) = ret {
//            Ok(DetectResult::open(dir.as_path(), ret.0.as_ref())?)
//        } else {
//            Ok(None)
//        }
//    }
//    pub async fn get_all_check_report(
//        &self,
//        _x: u32,
//        _y: u32,
//    ) -> anyhow::Result<BTreeMap<(u32, u32), DetectResult>> {
//        Ok(BTreeMap::new())
//    }
//
//    pub async fn get_last_water_report(
//        &self,
//        x: u32,
//        y: u32,
//    ) -> anyhow::Result<Option<WaterResult>> {
//        Ok(None)
//    }
//
//    pub async fn get_all_water_report(
//        &self,
//        _x: u32,
//        _y: u32,
//    ) -> anyhow::Result<BTreeMap<(u32, u32), WaterResult>> {
//        Ok(BTreeMap::new())
//    }
//
//    pub async fn auto_water(&self) -> anyhow::Result<bool> {
//        Ok(self.config.lock().await.auto_water)
//    }
//
//    pub async fn auto_check(&self) -> anyhow::Result<bool> {
//        Ok(self.config.lock().await.auto_check)
//    }
//    pub async fn set_auto_water(&self, state: bool) -> anyhow::Result<()> {
//        self.config.lock().await.auto_water = state;
//        self.sync_config().await?;
//        Ok(())
//    }
//    pub async fn set_auto_check(&self, state: bool) -> anyhow::Result<()> {
//        self.config.lock().await.auto_check = state;
//        self.sync_config().await?;
//        Ok(())
//    }
//
//    async fn capture(&self) -> anyhow::Result<DynamicImage> {
//        let mut this = self.inner.lock().await;
//        this.capture().await
//    }
//
//    pub async fn water_at(&self, x: u32, y: u32) -> anyhow::Result<WaterResult> {
//        let mut this = self.inner.lock().await;
//        this.goto(x, y).await?;
//        this.water(Duration::from_secs(2)).await?;
//        let img = this.capture().await?;
//        let time = std::time::SystemTime::now()
//            .duration_since(std::time::UNIX_EPOCH)?
//            .as_secs();
//        let ret = WaterResult {
//            x,
//            y,
//            image: img,
//            timestamp: time,
//        };
//        Ok(ret)
//    }
//
//    pub async fn capture_at(&self, x: u32, y: u32) -> anyhow::Result<CaptureResult> {
//        let mut this = self.inner.lock().await;
//        this.goto(x, y).await?;
//        let res = CaptureResult {
//            x,
//            y,
//            image: this.capture().await?,
//            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
//        };
//        Ok(res)
//    }
//
//    pub async fn check_at(&self, x: u32, y: u32) -> anyhow::Result<DetectResult> {
//        let CaptureResult {
//            x,
//            y,
//            mut image,
//            timestamp,
//        } = self.capture_at(x, y).await?;
//
//        let detections = self.detector.get_bounding_box(&image).await?;
//
//        let img_w = image.width();
//        let img_h = image.height();
//
//        for bbox in detections.iter() {
//            imageproc::drawing::draw_hollow_rect_mut(
//                &mut image,
//                imageproc::rect::Rect::at(*bbox.x() as i32, *bbox.y() as i32)
//                    .of_size(*bbox.w(), *bbox.h()),
//                image::Rgba([255, 0, 0, 100]),
//            )
//        }
//
//        let detection = detections
//            .iter()
//            .filter(|d| *d.x() < img_w / 2 && img_w / 2 < d.x() + d.w())
//            .find(|d| *d.y() < img_h / 2 && img_h / 2 < d.y() + d.h());
//
//        let ret = if let Some(detection) = detection {
//            let stage = match detection.object_id() {
//                0 => Stage::Young,
//                1 => Stage::Ready,
//                2 => Stage::Old,
//                3 => Stage::Empty,
//                4 => Stage::Unknown,
//                _ => Stage::Unknown,
//            };
//
//            DetectResult {
//                x,
//                y,
//                top: 20,
//                left: 20,
//                right: 20,
//                bottom: 20,
//                image,
//                stage,
//                timestamp,
//            }
//        } else {
//            DetectResult {
//                x,
//                y,
//                top: 20,
//                left: 20,
//                right: 20,
//                bottom: 20,
//                image,
//                stage: Stage::Unknown,
//                timestamp,
//            }
//        };
//        ret.save(&self.config.lock().await.data_dir)?;
//        Ok(ret)
//    }
//
//    pub async fn start_automation<T>(&self) -> anyhow::Result<Receiver<T>>
//    where
//        T: From<WaterResult> + From<DetectResult> + Send + Sync + 'static,
//    {
//        let (tx, rx) = bounded(100);
//
//        let this = self.clone();
//        async_std::task::spawn(async move {
//            loop {
//                if false {
//                    break;
//                }
//
//                let list_water: BTreeMap<(u32, u32), SystemTime>;
//                let mut last_check = UNIX_EPOCH;
//
//                let check_period = this.config.lock().await.auto_water_period;
//
//                let elapsed = SystemTime::now()
//                    .duration_since(last_check)
//                    .unwrap()
//                    .as_secs();
//
//                if elapsed <= check_period {
//                    for (x, y) in this.positions().await? {
//                        let exist = this
//                            .positions()
//                            .await?
//                            .iter()
//                            .any(|(px, py)| *px == x && *py == y);
//
//                        if exist && this.auto_water().await? {
//                            let res = this.check_at(x, y).await?;
//                            tx.send(res.into()).await?;
//                        }
//                    }
//                    last_check = SystemTime::now();
//                }
//                sleep(Duration::from_secs(1)).await;
//            }
//            Ok(()) as anyhow::Result<()>
//        });
//        Ok(rx)
//    }
//}
