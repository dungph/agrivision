use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_std::channel::bounded;

use crate::{
    camera::CameraIf,
    gateway::Gateway,
    linears::Linear2D,
    message::{InMsg, OutMsg, Stage},
    water::WaterIf,
    yolo::DetectionMachine,
};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub async fn run<L, W, D, C, P>(
    gateway: Gateway,
    positions: P,
    camera: C,
    detect: D,
    water: &mut W,
    linears: &mut L,
) -> anyhow::Result<()>
where
    L: Linear2D,
    W: WaterIf,
    D: DetectionMachine,
    C: CameraIf,
    P: IntoIterator<Item = [u32; 2]>,
{
    let auto_water = AtomicBool::new(false);
    let auto_check = AtomicBool::new(true);
    let capturing = AtomicBool::new(false);
    let watering = AtomicBool::new(false);
    let moving = AtomicBool::new(false);

    let pots: BTreeSet<_> = positions
        .into_iter()
        .map(|coor| (coor[0], coor[1]))
        .collect();

    let water_queue = bounded(1000);
    let check_queue = bounded(1000);
    let shutdown = AtomicBool::new(false);
    let actuator_task = async {
        loop {
            if shutdown.load(SeqCst) {
                break;
            }
            if let Ok((x, y)) = water_queue.1.try_recv() {
                gateway.send(OutMsg::ReportMoving(true)).await;
                moving.store(true, SeqCst);
                linears.goto(x, y).await?;
                gateway.send(OutMsg::ReportMoving(false)).await;
                moving.store(false, SeqCst);

                gateway.send(OutMsg::ReportWatering(true)).await;
                watering.store(true, SeqCst);
                water.water(Duration::from_secs(1)).await?;
                gateway.send(OutMsg::ReportWatering(false)).await;
                watering.store(false, SeqCst);
                gateway
                    .send(OutMsg::ReportWater {
                        x,
                        y,
                        timestamp: now(),
                    })
                    .await;
            } else if auto_water.load(SeqCst) {
                for pot in pots.iter() {
                    water_queue.0.try_send((pot.0, pot.1))?;
                }
            }

            if let Ok((x, y)) = check_queue.1.try_recv() {
                gateway.send(OutMsg::ReportMoving(true)).await;
                moving.store(true, SeqCst);
                linears.goto(x, y).await?;
                gateway.send(OutMsg::ReportMoving(false)).await;
                moving.store(false, SeqCst);

                gateway.send(OutMsg::ReportCapturing(true)).await;
                capturing.store(true, SeqCst);
                let mut img = camera.capture().await?;
                gateway.send(OutMsg::ReportCapturing(false)).await;
                capturing.store(false, SeqCst);

                let filename = format!(
                    "out/img_{x}_{y}_{}.jpg",
                    SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                let center_x = img.width() / 2;
                let center_y = img.height() / 2;

                let list_box = detect.get_bbox(&img).await?;

                for bbox in list_box.iter() {
                    imageproc::drawing::draw_hollow_rect_mut(
                        &mut img,
                        imageproc::rect::Rect::at(bbox.x as i32, bbox.y as i32)
                            .of_size(bbox.w, bbox.h),
                        image::Rgba([255, 0, 0, 100]),
                    )
                }
                img.save(&filename).ok();

                let report = list_box
                    .iter()
                    .filter(|b| b.point_in_box(center_x, center_y))
                    .map(|b| {
                        let state = match b.object_id {
                            0 => Stage::Young,
                            1 => Stage::Ready,
                            2 => Stage::Old,
                            _ => Stage::Unknown,
                        };
                        OutMsg::ReportCheck {
                            x,
                            y,
                            top: 20,
                            left: 20,
                            bottom: 20,
                            right: 20,
                            stage: state,
                            timestamp: now(),
                        }
                    })
                    .next()
                    .unwrap_or(OutMsg::ReportCheck {
                        x,
                        y,
                        top: 20,
                        left: 20,
                        bottom: 20,
                        right: 20,
                        stage: Stage::Unknown,
                        timestamp: now(),
                    });
                gateway.send(report).await;
                gateway
                    .send(OutMsg::ReportPotImageFile {
                        x,
                        y,
                        file_path: filename.clone(),
                    })
                    .await;
            } else if auto_check.load(SeqCst) {
                for pot in pots.iter() {
                    check_queue.0.try_send((pot.0, pot.1))?;
                }
            }
            async_std::task::sleep(Duration::from_secs(3)).await;
        }
        Ok(()) as anyhow::Result<()>
    };

    let handle_task = async {
        loop {
            let inmsg = gateway.recv().await;
            match inmsg {
                InMsg::GetReport => {
                    gateway.send_myself(InMsg::GetListPot).await;
                    gateway.send_myself(InMsg::GetMovingState).await;
                    gateway.send_myself(InMsg::GetWateringState).await;
                    gateway.send_myself(InMsg::GetCapturingState).await;
                    gateway.send_myself(InMsg::GetAutoWater).await;
                    gateway.send_myself(InMsg::GetAutoCheck).await;
                }
                InMsg::GetListPot => {
                    for p in pots.iter() {
                        gateway.send(OutMsg::ReportPot { x: p.0, y: p.1 }).await;
                    }
                }
                InMsg::GetMovingState => {
                    gateway
                        .send(OutMsg::ReportMoving(moving.load(SeqCst)))
                        .await;
                }
                InMsg::GetWateringState => {
                    gateway
                        .send(OutMsg::ReportWatering(watering.load(SeqCst)))
                        .await;
                }
                InMsg::GetCapturingState => {
                    gateway
                        .send(OutMsg::ReportCapturing(capturing.load(SeqCst)))
                        .await;
                }
                InMsg::GetAutoWater => {
                    gateway
                        .send(OutMsg::ReportAutoWater(auto_water.load(SeqCst)))
                        .await;
                }
                InMsg::GetAutoCheck => {
                    gateway
                        .send(OutMsg::ReportAutoCheck(auto_check.load(SeqCst)))
                        .await;
                }
                InMsg::SetAutoWater(state) => {
                    auto_water.store(state, SeqCst);
                    gateway.send(OutMsg::ReportAutoWater(state)).await;
                }
                InMsg::SetAutoCheck(state) => {
                    auto_check.store(state, SeqCst);
                    gateway.send(OutMsg::ReportAutoCheck(state)).await;
                }
                InMsg::Shutdown => {
                    shutdown.store(true, SeqCst);
                    break;
                }
                InMsg::Water { x, y } => {
                    water_queue.0.send((x, y)).await.ok();
                }
                InMsg::Check { x, y } => {
                    check_queue.0.send((x, y)).await.ok();
                }
            }
        }
        Ok(()) as anyhow::Result<()>
    };

    futures_lite::future::or(actuator_task, handle_task).await
}
