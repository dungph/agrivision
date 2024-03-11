use clap::Parser;
use config::Config;
use image::DynamicImage;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use toml_edit::{value, Document};

pub mod camera;
pub mod config;
pub mod control;
pub mod gateway;
pub mod linears;
pub mod message;
pub mod water;
pub mod yolo;

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
enum Task {
    Run {
        #[arg(short, long, default_value = "./config.toml")]
        config_path: PathBuf,
    },
    RunTest {
        #[arg(short, long, default_value = "./config.toml")]
        config_path: PathBuf,
    },
    Template,
    ProcessImage {
        #[arg(short, long, default_value = "./best.safetensors")]
        model_path: PathBuf,

        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long, default_value = "./out.png")]
        output: PathBuf,
    },
    ExtractImage {
        #[arg(short, long, default_value = "./best.safetensors")]
        model_path: PathBuf,

        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long, default_value = "./out/")]
        output: PathBuf,
    },
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    match Task::parse() {
        Task::Run { config_path } => {
            let config = Config::open(&config_path)?;
            let mut linears = linears::Linears::new(&config)?;
            let mut water = water::Water::new(config.watering())?;
            let yolo = yolo::Yolo::new(config.yolo())?;
            let camera = camera::Camera::from_path(config.camera().video_path())?;
            let gateway = gateway::Gateway::with_socket(*config.http().listen_socket());
            let pos = config.positions().positions().clone();
            control::run(gateway, pos, camera, yolo, &mut water, &mut linears).await?;
        }
        Task::RunTest { config_path } => {
            let mut linears = linears::DummyLinear2D;
            let mut water = water::DummyWater;
            let yolo = yolo::DummyDetectionMachine;
            let camera = camera::DummyCamera;
            let gateway = gateway::Gateway::with_socket("0.0.0.0:8080".parse().unwrap());
            let pos = vec![
                [0, 0],
                [0, 100],
                [80, 220],
                [100, 0],
                [100, 100],
                [200, 0],
                [200, 100],
                [300, 0],
                [300, 100],
            ];
            control::run(gateway, pos, camera, yolo, &mut water, &mut linears).await?;
        }
        Task::Template => {
            let s = toml::to_string_pretty(&Config::default()).unwrap();
            let mut doc = s.parse::<Document>()?;

            for i in ["linear_x", "linear_y"] {
                for j in ["en_pin", "dir_pin", "step_pin"] {
                    doc[i][j] = value(doc[i][j].as_table().unwrap().clone().into_inline_table());
                }
            }
            doc["watering"]["pin"] = value(
                doc["watering"]["pin"]
                    .as_table()
                    .unwrap()
                    .clone()
                    .into_inline_table(),
            );

            println!("{}", doc);
        }
        Task::ProcessImage {
            model_path,
            input,
            output,
        } => {
            let yolo = yolo::Yolo::open(&model_path, config::ModelSize::N)?;

            let mut image = image::open(input)?;
            yolo.draw_bounding_box(&mut image).await?;

            image.save(output)?;
        }
        Task::ExtractImage {
            model_path,
            input,
            output,
        } => {
            let yolo = yolo::Yolo::open(&model_path, config::ModelSize::N)?;
            if output.is_dir() {
                if input.is_dir() {
                    let mut id = 0;
                    let files = std::fs::read_dir(&input)?;
                    for file in files {
                        let file = file?;
                        let path = file.path();
                        if let Ok(img) = image::open(path) {
                            if let Ok(outs) = extract_image(&yolo, &img).await {
                                for out in outs {
                                    let file_name = format!("out{:04}.png", id);
                                    id += 1;
                                    out.save(output.as_path().join(file_name))?;
                                }
                            }
                        }
                    }
                } else if input.is_file() {
                    let mut id = 0;
                    if let Ok(img) = image::open(&input) {
                        if let Ok(outs) = extract_image(&yolo, &img).await {
                            for out in outs {
                                let file_name = format!("out{:04}.png", id);
                                id += 1;
                                out.save(output.as_path().join(file_name))?;
                            }
                        }
                    }
                }
            } else {
                println!("output is not a dir");
            }
        }
    }

    Ok(())
}

async fn extract_image(
    model: &yolo::Yolo,
    img: &DynamicImage,
) -> anyhow::Result<Vec<DynamicImage>> {
    let bboxes = model.get_bounding_box(img).await?;
    let mut ret = Vec::new();
    for bbox in bboxes {
        let s = bbox.w.max(bbox.h);
        let padding = s / 20;
        let x = bbox.x;
        let y = bbox.y;
        if x > padding && y > padding && x + padding < img.width() && y + padding < img.height() {
            let img = img.crop_imm(x - padding, y - padding, s + padding, s + padding);
            ret.push(img.resize_exact(640, 640, image::imageops::FilterType::Gaussian));
        }
    }
    Ok(ret)
}
