use std::path::PathBuf;

use clap::Parser;
use simple_logger::SimpleLogger;

mod client;
mod database;
mod system;

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Task {
    #[arg(short, long, default_value = "./config.toml")]
    config_path: PathBuf,

    #[arg(short, long, default_value = "false")]
    template: bool,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let cmd = Task::parse();

    if cmd.template {
        println!(
            "{}",
            toml::to_string(&system::LocalSystemConfig::default())?
        );
        return Ok(());
    }
    database::migrate().await?;
    system::init(&cmd.config_path).await?;
    system::start_automation().await;
    client::start_http().await?;

    Ok(())
}
