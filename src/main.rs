use clap::Parser;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

pub mod actuator;
pub mod capture;
pub mod control;
pub mod gateway;
pub mod http;
pub mod message;
pub mod settings;
pub mod vision;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    config: PathBuf,
    #[arg(short, long)]
    template: bool,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    if args.template {
        settings::set_config(args.config);
        settings::save().unwrap();
        return Ok(());
    }

    settings::open(&args.config).unwrap();

    http::start_server().await;
    capture::start_capture().await;
    vision::start_vision().await;
    actuator::start_linear().await;

    //if let Some(pump_pin) = args.pump_pin {
    //    actuator::start_pump(
    //        pump_pin,
    //        Duration::from_secs(args.pump_on),
    //        Duration::from_secs(args.pump_off),
    //    )
    //    .await?;
    //}

    std::future::pending().await
}
