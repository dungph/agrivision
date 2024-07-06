mod actuator;
mod camera;
mod detector;

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_std::sync::Mutex;
use async_std::task::sleep;
use derive_getters::Getters;
use image::{DynamicImage, ImageFormat, Rgb};
use serde::{Deserialize, Serialize};

use crate::database::{self};

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
    pub timestamp: i64,
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
pub fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs() as i64
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
        timestamp: timestamp(),
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

pub async fn recheck_id(check_id: i64) -> anyhow::Result<()> {
    let mut check = database::query_check(check_id).await.map_err(|e| dbg!(e))?;
    let image_row = database::query_image(check.image_id)
        .await
        .map_err(|e| dbg!(e))?;

    let image = image::load_from_memory(&image_row.image).map_err(|e| dbg!(e))?;

    let detection = DETECTOR
        .lock()
        .await
        .as_mut()
        .unwrap()
        .detect(&image)
        .await
        .map_err(|e| dbg!(e))
        .map_err(|e| dbg!(e))?;

    let image = imageproc::drawing::draw_hollow_rect(
        &image.to_rgb8(),
        imageproc::rect::Rect::at(detection.x as i32, detection.y as i32)
            .of_size(detection.width, detection.height),
        image::Rgb([255u8, 0, 0]),
    );
    println!("{}", detection.class);

    let image = imageproc::drawing::draw_hollow_circle(
        &image,
        (image.width() as i32 / 2, image.height() as i32 / 2),
        50,
        Rgb([127, 127, 127]),
    );

    let mut img = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut img), ImageFormat::Jpeg)
        .map_err(|e| dbg!(e))?;
    database::update_image(image_row.id, &img)
        .await
        .map_err(|e| dbg!(e))?;

    let stage = database::query_stages(None, Some(&detection.class))
        .await
        .map_err(|e| dbg!(e))?
        .pop();

    let stage = if let Some(stage) = stage {
        stage
    } else {
        database::upsert_stage(database::StageData {
            id: 0,
            stage: detection.class.to_owned(),
            first_stage: false,
            water_period: 1000,
            check_period: 1000,
            water_duration: 1,
        })
        .await
        .map_err(|e| dbg!(e))?
    };

    check.stage_id = stage.id;
    database::upsert_check(check).await.map_err(|e| dbg!(e))?;
    Ok(())
}

pub async fn check_at(position_id: i64, force_water: bool) -> anyhow::Result<()> {
    let position = database::query_position(Some(position_id), None)
        .await?
        .pop();
    if position.is_none() {
        return Ok(());
    }
    let position = position.unwrap();

    let capture = capture_at(position.x as u32, position.y as u32)
        .await
        .map_err(|e| dbg!(e))?;
    let image = capture.image;
    let created_ts = capture.timestamp;
    let edge = image.height().min(image.width());

    let image = image.crop_imm(
        (image.width() - edge) / 2,
        (image.height() - edge) / 2,
        edge,
        edge,
    );
    //.resize(640, 640, image::imageops::FilterType::Gaussian);

    let detection = DETECTOR
        .lock()
        .await
        .as_mut()
        .unwrap()
        .detect(&image)
        .await
        .map_err(|e| dbg!(e))?;

    let image = imageproc::drawing::draw_hollow_rect(
        &image.to_rgb8(),
        imageproc::rect::Rect::at(detection.x as i32, detection.y as i32)
            .of_size(detection.width, detection.height),
        image::Rgb([255u8, 0, 0]),
    );
    println!("{}", detection.class);

    let image = imageproc::drawing::draw_hollow_circle(
        &image,
        (image.width() as i32 / 2, image.height() as i32 / 2),
        50,
        Rgb([127, 127, 127]),
    );

    let mut img = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut img), ImageFormat::Jpeg)
        .map_err(|e| dbg!(e))?;
    let image_id = database::insert_image(&img).await.map_err(|e| dbg!(e))?;

    let stage = database::query_stages(None, Some(&detection.class))
        .await?
        .pop();

    let stage = if let Some(stage) = stage {
        stage
    } else {
        database::upsert_stage(database::StageData {
            id: 0,
            stage: detection.class.to_owned(),
            first_stage: false,
            water_period: 1000,
            check_period: 1000,
            water_duration: 1,
        })
        .await?
    };

    let check_id = database::upsert_check(database::CheckData {
        id: 0,
        position_id: position.id,
        created_ts,
        stage_id: stage.id,
        image_id,
        watered: false,
    })
    .await?;

    let last_water = database::query_last_checks(Some(position.id), true)
        .await?
        .pop();
    match last_water {
        Some(last_water)
            if (last_water.created_ts + stage.water_period > timestamp()) && !force_water => {}
        _ => {
            water_at(
                position.x as u32,
                position.y as u32,
                Duration::from_secs(stage.water_duration as u64),
            )
            .await?;
            database::upsert_check(database::CheckData {
                id: check_id,
                position_id: position.id,
                created_ts,
                stage_id: stage.id,
                image_id,
                watered: true,
            })
            .await?;
        }
    }

    sync_profile().await?;
    Ok(())
}

pub async fn start_automation() {
    async_std::task::spawn(async move {
        loop {
            if false {
                break;
            }

            let positions = database::query_position(None, None).await?;
            log::info!("iterating {} positions", positions.len());

            let mut handlers = Vec::new();

            for pos in positions {
                let handler = async_std::task::spawn(async move {
                    let last_check = database::query_last_checks(Some(pos.id), false)
                        .await?
                        .pop();
                    if let Some(last_check) = last_check {
                        let stage = database::query_stages(Some(last_check.stage_id), None)
                            .await?
                            .pop();
                        if let Some(stage) = stage {
                            if last_check.created_ts + stage.check_period <= timestamp() {
                                check_at(pos.id, false).await?;
                            }
                        }
                    } else {
                        check_at(pos.id, true).await?;
                    }

                    Ok(()) as anyhow::Result<()>
                });
                handlers.push(handler);
            }
            for handler in handlers {
                handler.await.ok();
            }

            sleep(Duration::from_secs(1)).await;
        }
        Ok(()) as anyhow::Result<()>
    });
}
