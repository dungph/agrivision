use std::path::PathBuf;

use clap::Parser;
use simple_logger::SimpleLogger;

mod client;
mod system;

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
enum Task {
    Run {
        #[arg(short, long, default_value = "./config.toml")]
        config_path: PathBuf,
    },
    Template,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    match Task::parse() {
        Task::Run { config_path } => {
            let system = system::System::from_path(&config_path)?;

            client::start_http(system).await?;
        }
        Task::Template => {
            println!(
                "{}",
                toml::to_string(&system::LocalSystemConfig::default())?
            );
        } //Task::ProcessImage {
          //    model_path,
          //    input,
          //    output,
          //} => {
          //    let yolo = detector::Yolov8::open(&model_path, config::ModelSize::N)?;

          //    let mut image = image::open(input)?;
          //    yolo.draw_bounding_box(&mut image).await?;

          //    image.save(output)?;
          //}
          //Task::ExtractImage {
          //    model_path,
          //    input,
          //    output,
          //} => {
          //    let yolo = detector::Yolov8::open(&model_path, config::ModelSize::N)?;
          //    if output.is_dir() {
          //        if input.is_dir() {
          //            let mut id = 0;
          //            let files = std::fs::read_dir(&input)?;
          //            for file in files {
          //                let file = file?;
          //                let path = file.path();
          //                if let Ok(img) = image::open(path) {
          //                    if let Ok(outs) = extract_image(&yolo, &img).await {
          //                        for out in outs {
          //                            let file_name = format!("out{:04}.png", id);
          //                            id += 1;
          //                            out.save(output.as_path().join(file_name))?;
          //                        }
          //                    }
          //                }
          //            }
          //        } else if input.is_file() {
          //            let mut id = 0;
          //            if let Ok(img) = image::open(&input) {
          //                if let Ok(outs) = extract_image(&yolo, &img).await {
          //                    for out in outs {
          //                        let file_name = format!("out{:04}.png", id);
          //                        id += 1;
          //                        out.save(output.as_path().join(file_name))?;
          //                    }
          //                }
          //            }
          //        }
          //    } else {
          //        println!("output is not a dir");
          //    }
          //}
    }

    Ok(())
}

//async fn extract_image(
//    model: &detector::Yolov8,
//    img: &DynamicImage,
//) -> anyhow::Result<Vec<DynamicImage>> {
//    let bboxes = model.get_bounding_box(img).await?;
//    let mut ret = Vec::new();
//    for bbox in bboxes {
//        let s = bbox.w.max(bbox.h);
//        let padding = s / 20;
//        let x = bbox.x;
//        let y = bbox.y;
//        if x > padding && y > padding && x + padding < img.width() && y + padding < img.height() {
//            let img = img.crop_imm(x - padding, y - padding, s + padding, s + padding);
//            ret.push(img.resize_exact(640, 640, image::imageops::FilterType::Gaussian));
//        }
//    }
//    Ok(ret)
//}
