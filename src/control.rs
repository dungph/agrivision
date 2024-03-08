use std::{
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use async_std::sync::Mutex;

use crate::{
    camera::Camera,
    config::Config,
    gateway::Gateway,
    linears::Linears,
    message::{Message, Pot, State},
    water::Water,
    yolo::Yolo,
};

pub struct Controller {
    linears: Arc<Mutex<Linears>>,
    water: Arc<Mutex<Water>>,
    pots: Arc<Mutex<Vec<Pot>>>,
    auto_water: Arc<AtomicBool>,
    auto_check: Arc<AtomicBool>,
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
            .map(|coor| Pot {
                x: coor.0,
                y: coor.1,
                top: 20,
                left: 20,
                bottom: 20,
                right: 20,
                stage: State::Unknown,
                timestamp: 0,
            })
            .collect();
        Ok(Self {
            linears: Arc::new(Mutex::new(linears)),
            water: Arc::new(Mutex::new(water)),
            yolo: Arc::new(vision),
            camera: Arc::new(camera),
            pots: Arc::new(Mutex::new(pots)),
            auto_water: Arc::new(AtomicBool::new(false)),
            auto_check: Arc::new(AtomicBool::new(false)),
            gateway: Arc::new(gateway),
        })
    }
    async fn pots(&self) -> Vec<Pot> {
        self.pots.lock().await.clone()
    }
    async fn water(&self, x: u32, y: u32) -> anyhow::Result<()> {
        let mut linears = self.linears.lock().await;
        linears.goto(x as i32, y as i32).await?;
        self.water.lock().await.water().await?;
        Ok(())
    }
    async fn check(&self, x: u32, y: u32) -> anyhow::Result<()> {
        let mut linears = self.linears.lock().await;
        linears.goto(x as i32, y as i32).await?;
        let img = self.camera.capture().await?;

        let center_x = img.width() / 2;
        let center_y = img.height() / 2;

        let list_box = self.yolo.get_bounding_box(&img).await?;

        let pot = list_box
            .iter()
            .filter(|b| b.point_in_box(center_x, center_y))
            .map(|b| {
                let state = match b.object_id {
                    0 => State::Young,
                    1 => State::Ready,
                    2 => State::Old,
                    _ => State::Unknown,
                };
                Pot {
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
            .unwrap_or(Pot {
                x,
                y,
                top: 20,
                left: 20,
                bottom: 20,
                right: 20,
                stage: State::Unknown,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        self.gateway.send(Message::ReportPot(pot)).await;
        Ok(())
    }
    pub async fn process_incoming(&self) -> anyhow::Result<()> {
        loop {
            match self.gateway.recv().await {
                Message::GetListPot => {
                    for pot in self.pots().await.iter() {
                        self.gateway.send(Message::ReportPot(pot.clone())).await;
                    }
                }
                Message::AutoWater(s) => {
                    self.auto_water.store(s, SeqCst);
                }
                Message::ManualWater { x, y } => {
                    self.water(x, y).await?;
                }
                Message::AutoCheck(s) => {
                    self.auto_check.store(s, SeqCst);
                }
                Message::Check { x, y } => {
                    self.check(x, y).await?;
                }
                Message::ReportPot(_) => todo!(),
                Message::Status(_) => todo!(),
                Message::Error(_) => todo!(),
            }
        }
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        loop {
            if self.auto_water.load(SeqCst) {
                for pot in self.pots().await.iter() {
                    self.water(pot.x, pot.y).await?;
                }
            }
            if self.auto_check.load(SeqCst) {
                for pot in self.pots().await.iter() {
                    self.check(pot.x, pot.y).await?;
                }
            }
        }
    }
}
