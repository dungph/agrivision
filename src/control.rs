use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_std::sync::Mutex;

use crate::{
    camera::Camera,
    config::Config,
    gateway::Gateway,
    linears::Linears,
    message::{Message, Stage},
    water::Water,
    yolo::Yolo,
};

pub struct Controller {
    linears: Arc<Mutex<Linears>>,
    water: Arc<Mutex<Water>>,
    pots: Arc<Mutex<BTreeSet<(u32, u32)>>>,
    auto_water: Arc<AtomicBool>,
    auto_check: Arc<AtomicBool>,
    capturing: AtomicBool,
    watering: AtomicBool,
    moving: AtomicBool,
    gateway: Arc<Gateway>,
    yolo: Arc<Yolo>,
    camera: Arc<Camera>,
}

impl Controller {
    pub fn with_config(config: &Config) -> anyhow::Result<Self> {
        let linears = Linears::new(config)?;
        let water = Water::new(config.watering())?;
        let vision = Yolo::new(config.yolo())?;
        let gateway = Gateway::with_socket(*config.http().listen_socket());
        let camera = Camera::from_path(config.camera().video_path())?;

        let pots = config
            .positions()
            .positions()
            .iter()
            .map(|coor| (coor[0], coor[1]))
            .collect();
        Ok(Self {
            linears: Arc::new(Mutex::new(linears)),
            water: Arc::new(Mutex::new(water)),
            yolo: Arc::new(vision),
            camera: Arc::new(camera),
            pots: Arc::new(Mutex::new(pots)),
            auto_water: Arc::new(AtomicBool::new(true)),
            auto_check: Arc::new(AtomicBool::new(true)),
            gateway: Arc::new(gateway),
            capturing: AtomicBool::new(false),
            watering: AtomicBool::new(false),
            moving: AtomicBool::new(false),
        })
    }
    async fn send(&self, msg: Message) {
        self.gateway.send(msg).await;
    }
    async fn send_myself(&self, msg: Message) {
        self.gateway.send_myself(msg).await;
    }
    async fn pots(&self) -> BTreeSet<(u32, u32)> {
        self.pots.lock().await.clone()
    }
    async fn goto(&self, x: u32, y: u32) -> anyhow::Result<()> {
        let mut linears = self.linears.lock().await;
        self.send(Message::ReportMoving(true)).await;
        self.moving.store(true, SeqCst);
        linears.goto(x as i32, y as i32).await?;
        self.moving.store(false, SeqCst);
        self.send(Message::ReportMoving(false)).await;
        Ok(())
    }
    async fn water(&self) -> anyhow::Result<()> {
        self.send(Message::ReportWatering(true)).await;
        self.watering.store(true, SeqCst);
        self.water.lock().await.water().await?;
        self.watering.store(false, SeqCst);
        self.send(Message::ReportWatering(false)).await;
        Ok(())
    }
    async fn goto_water(&self, x: u32, y: u32) -> anyhow::Result<()> {
        self.goto(x, y).await?;
        self.water().await?;
        self.send(Message::ReportWater {
            x,
            y,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
        .await;
        Ok(())
    }
    async fn goto_check(&self, x: u32, y: u32) -> anyhow::Result<()> {
        self.goto(x, y).await?;

        self.send(Message::ReportCapturing(true)).await;
        self.capturing.store(true, SeqCst);
        let img = self.camera.capture().await?;
        self.send(Message::ReportCapturing(false)).await;
        self.capturing.store(false, SeqCst);

        let filename = format!(
            "out/img_{x}_{y}_{}.jpg",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        img.save(&filename).ok();
        self.send(Message::ReportPotImageFile {
            x,
            y,
            file_path: filename,
        })
        .await;

        let center_x = img.width() / 2;
        let center_y = img.height() / 2;

        let list_box = self.yolo.get_bounding_box(&img).await?;

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
                Message::ReportCheck {
                    x,
                    y,
                    top: 20,
                    left: 20,
                    bottom: 20,
                    right: 20,
                    stage: state,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                }
            })
            .next()
            .unwrap_or(Message::ReportCheck {
                x,
                y,
                top: 20,
                left: 20,
                bottom: 20,
                right: 20,
                stage: Stage::Unknown,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        self.gateway.send(report).await;
        Ok(())
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        loop {
            self.send_myself(Message::GetAutoWater).await;
            if self.auto_water.load(SeqCst) {
                for pot in self.pots().await.iter() {
                    self.gateway
                        .send_myself(Message::ManualWater { x: pot.0, y: pot.1 })
                        .await;
                }
            }
            self.send_myself(Message::GetAutoCheck).await;
            if self.auto_check.load(SeqCst) {
                for pot in self.pots().await.iter() {
                    self.gateway
                        .send_myself(Message::ManualCheck { x: pot.0, y: pot.1 })
                        .await;
                }
            }
            async_std::task::sleep(Duration::from_secs(3)).await;
        }
    }
    pub async fn process_incoming(&self) -> anyhow::Result<()> {
        loop {
            match self.gateway.recv().await {
                Message::GetListPot => {
                    for pot in self.pots().await.iter() {
                        self.gateway
                            .send(Message::ReportPot { x: pot.0, y: pot.1 })
                            .await;
                    }
                }
                Message::AutoWater(s) => {
                    self.auto_water.store(s, SeqCst);
                }
                Message::ManualWater { x, y } => {
                    self.goto_water(x, y).await?;
                }
                Message::AutoCheck(s) => {
                    self.auto_check.store(s, SeqCst);
                }
                Message::ManualCheck { x, y } => {
                    self.goto_check(x, y).await?;
                }
                Message::GetReport => {
                    self.send_myself(Message::GetListPot).await;
                    self.send_myself(Message::GetMovingState).await;
                    self.send_myself(Message::GetWateringState).await;
                    self.send_myself(Message::GetCapturingState).await;
                    self.send_myself(Message::GetAutoWater).await;
                    self.send_myself(Message::GetAutoCheck).await;
                }
                Message::GetMovingState => {
                    self.send(Message::ReportMoving(self.moving.load(SeqCst)))
                        .await;
                }
                Message::GetWateringState => {
                    self.send(Message::ReportWatering(self.watering.load(SeqCst)))
                        .await;
                }
                Message::GetCapturingState => {
                    self.send(Message::ReportCapturing(self.capturing.load(SeqCst)))
                        .await;
                }
                Message::GetAutoWater => {
                    self.send(Message::ReportAutoWater(self.auto_water.load(SeqCst)))
                        .await;
                }
                Message::GetAutoCheck => {
                    self.send(Message::ReportAutoCheck(self.auto_check.load(SeqCst)))
                        .await;
                }
                Message::Status(_) => todo!(),
                Message::Error(_) => todo!(),
                Message::ReportPot { x, y } => todo!(),
                Message::ReportMoving(_) => todo!(),
                Message::ReportAutoWater(_) => todo!(),
                Message::ReportWater { x, y, timestamp } => todo!(),
                Message::ReportWatering(_) => todo!(),
                Message::ReportAutoCheck(_) => todo!(),
                Message::ReportCheck {
                    x,
                    y,
                    top,
                    left,
                    bottom,
                    right,
                    stage,
                    timestamp,
                } => todo!(),
                Message::ReportCapturing(_) => todo!(),
            }
        }
    }
}
