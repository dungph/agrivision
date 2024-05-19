mod actuator;
mod camera;
mod detector;

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_std::sync::Mutex;
use async_std::task::sleep;
use derive_getters::Getters;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};

use crate::database::{self, CheckResult};

use self::actuator::ActuatorProfile;
use self::camera::CameraConfig;
use detector::DetectorConfig;

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

static CONFIG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static ACTUATOR: Mutex<Option<ActuatorProfile>> = Mutex::new(None);
static CAMERA: Mutex<Option<CameraConfig>> = Mutex::new(None);
static DETECTOR: Mutex<Option<DetectorConfig>> = Mutex::new(None);

#[derive(Default, Serialize, Deserialize)]
pub struct LocalSystemConfig {
    actuators: ActuatorProfile,
    detector: DetectorConfig,
    camera: CameraConfig,
}

pub async fn init(config_path: &Path) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(config_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let config: LocalSystemConfig = toml::from_str(&data)?;
    ACTUATOR.lock().await.replace(config.actuators);
    DETECTOR.lock().await.replace(config.detector);
    CAMERA.lock().await.replace(config.camera);
    CONFIG_PATH.lock().await.replace(config_path.to_owned());
    Ok(())
}

async fn sync_profile() -> Result<(), anyhow::Error> {
    let actuators = ACTUATOR.lock().await.clone();
    let detector = DETECTOR.lock().await.clone();
    let camera = CAMERA.lock().await.clone();
    let config = LocalSystemConfig {
        actuators: actuators.unwrap(),
        detector: detector.unwrap(),
        camera: camera.unwrap(),
    };

    let conf_data = toml::to_string(&config)?;

    let mut file = std::fs::File::options()
        .write(true)
        .truncate(true)
        .open(CONFIG_PATH.lock().await.clone().unwrap())?;
    file.write_all(conf_data.as_bytes())?;
    Ok(())
}

pub async fn capture_raw() -> anyhow::Result<Vec<u8>> {
    let mut camera = CAMERA.lock().await;
    camera.as_mut().unwrap().capture_raw().await
}
pub async fn capture_raw_at(x: u32, y: u32) -> anyhow::Result<Vec<u8>> {
    let mut ac = ACTUATOR.lock().await;
    ac.as_mut().unwrap().goto(x, y).await?;
    capture_raw().await
}

pub async fn capture_at(x: u32, y: u32) -> anyhow::Result<CaptureResult> {
    let mut ac = ACTUATOR.lock().await;
    ac.as_mut().unwrap().goto(x, y).await?;
    let mut camera = CAMERA.lock().await;
    let image = camera.as_mut().unwrap().capture().await?;
    drop(ac);
    drop(camera);

    let res = CaptureResult {
        x,
        y,
        image,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };

    sync_profile().await?;
    Ok(res)
}
async fn water_at(x: u32, y: u32, dur: Duration) -> anyhow::Result<()> {
    let mut ac = ACTUATOR.lock().await;
    ac.as_mut()
        .expect("init must be called")
        .water_at(x, y, dur)
        .await
        .map_err(|e| dbg!(e))?;
    Ok(())
}

pub async fn check_at(x: u32, y: u32, force_water: bool) -> anyhow::Result<CheckResult> {
    let capture = capture_at(x, y).await.map_err(|e| dbg!(e))?;
    let image = capture.image;
    let timestamp = capture.timestamp;

    let detection = DETECTOR
        .lock()
        .await
        .as_mut()
        .unwrap()
        .detect(&image)
        .await
        .map_err(|e| dbg!(e))?;

    let mut img = Vec::new();
    image.write_to(&mut Cursor::new(&mut img), ImageFormat::Jpeg)?;

    let mut ret = database::CheckResult {
        x,
        y,
        image_id: database::insert_image(&img).await.map_err(|e| dbg!(e))?,
        stage: detection.class,
        timestamp,
        water_duration: None,
    };

    let last_water = database::get_last_water(x, y).await?;
    let config = database::get_checking_config_stage(&last_water.stage).await?;
    let water_dur = config.water_duration;

    let timestamp_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();

    if last_water.timestamp + config.water_period as u64 <= timestamp_now || force_water {
        water_at(x, y, Duration::from_secs(water_dur as u64))
            .await
            .map_err(|e| dbg!(e))?;
        ret.water_duration = Some(water_dur as u32);
    }

    database::insert_check(&ret).await.map_err(|e| dbg!(e))?;

    sync_profile().await.map_err(|e| dbg!(e))?;
    Ok(ret)
}

pub async fn start_automation() {
    async_std::task::spawn(async move {
        loop {
            if false {
                break;
            }

            let positions = database::get_list_position().await?;
            log::info!("iterating {} positions", positions.len());

            for pos in positions {
                let last_check = database::get_last_check(pos.x, pos.y).await?;
                let config = database::get_checking_config_stage(&last_check.stage).await?;

                let timestamp_now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs();

                if last_check.timestamp + config.check_period as u64 <= timestamp_now {
                    check_at(pos.x, pos.y, false).await?;
                }
            }

            sleep(Duration::from_secs(10)).await;
        }
        Ok(()) as anyhow::Result<()>
    });
}
