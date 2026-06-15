mod cli;
mod config;
mod processor;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "vidspeed")]
#[command(about = "Speed up videos without sound", long_about = None)]
struct Args {
    /// Input video file
    input: String,

    /// Output video file (optional)
    #[arg(short, long)]
    output: Option<String>,

    /// Speed factor (1.0 = normal, >1 faster, <1 slower)
    #[arg(short, long, default_value_t = 2.0)]
    speed: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "vidspeed=info".to_string()))
        .with_target(false)
        .init();

    let args = Args::parse();

    cli::process_video(&args.input, args.output.as_deref(), args.speed).await?;

    Ok(())
}
