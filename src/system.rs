mod actuator;
mod detector;

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_std::sync::Mutex;
use async_std::task::sleep;
use derive_getters::Getters;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::database::{self, CheckResult};

use self::actuator::ActuatorProfile;
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
static DETECTOR: Mutex<Option<DetectorConfig>> = Mutex::new(None);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalSystemConfig {
    actuators: ActuatorProfile,
    detector: DetectorConfig,
}

pub async fn init(config_path: &Path) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(config_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let config: LocalSystemConfig = toml::from_str(&data)?;
    ACTUATOR.lock().await.replace(config.actuators);
    DETECTOR.lock().await.replace(config.detector);
    CONFIG_PATH.lock().await.replace(config_path.to_owned());
    Ok(())
}

async fn sync_profile() -> Result<(), anyhow::Error> {
    let actuators = ACTUATOR.lock().await.clone();
    let detector = DETECTOR.lock().await.clone();
    let config = LocalSystemConfig {
        actuators: actuators.unwrap(),
        detector: detector.unwrap(),
    };

    let conf_data = toml::to_string(&config)?;

    let mut file = std::fs::File::options()
        .write(true)
        .truncate(true)
        .open(CONFIG_PATH.lock().await.clone().unwrap())?;
    file.write_all(conf_data.as_bytes())?;
    Ok(())
}

pub async fn capture_at(x: u32, y: u32) -> anyhow::Result<CaptureResult> {
    let mut ac = ACTUATOR.lock().await;
    let image = ac
        .as_mut()
        .unwrap()
        .capture_at(x, y)
        .await
        .map_err(|e| dbg!(e))?;

    let res = CaptureResult {
        x,
        y,
        image,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };
    Ok(res)
}
async fn water_at(x: u32, y: u32, dur: Duration) -> anyhow::Result<()> {
    let mut ac = ACTUATOR.lock().await;
    ac.as_mut()
        .expect("init must be called")
        .water_at(x, y, dur)
        .await?;
    Ok(())
}

pub async fn check_at(x: u32, y: u32, force_water: bool) -> anyhow::Result<CheckResult> {
    let capture = capture_at(x, y).await.map_err(|e| dbg!(e))?;
    let image = capture.image;
    let timestamp = capture.timestamp;

    let detection = {
        let mut detector = DETECTOR.lock().await;
        detector
            .as_mut()
            .unwrap()
            .detect(&image)
            .await
            .map_err(|e| dbg!(e))?
    };

    let mut img = Vec::new();
    image.write_to(&mut Cursor::new(&mut img), image::ImageFormat::Jpeg)?;

    let mut ret = database::CheckResult {
        x,
        y,
        image: img,
        stage: detection.class,
        timestamp,
        water_duration: None,
    };

    if let Some(duration) = database::shoud_water(x, y).await? {
        water_at(x, y, Duration::from_secs(duration)).await?;
        ret.water_duration = Some(duration as u32);
    } else if force_water {
        let dur = database::water_duration(&ret.stage).await?;
        water_at(x, y, dur).await?;
        ret.water_duration = Some(dur.as_secs() as u32);
    }

    database::insert_check(&ret).await.unwrap();

    sync_profile().await?;
    Ok(ret)
}

pub async fn start_automation() {
    async_std::task::spawn(async move {
        loop {
            if false {
                break;
            }

            let positions = database::list_position("admin").await?;

            for pos in positions {
                if database::should_check(pos.x, pos.y).await? {
                    check_at(pos.x, pos.y, false).await?;
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
        Ok(()) as anyhow::Result<()>
    });
}
