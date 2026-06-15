mod cli;
mod processor;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "vidspeed")]
#[command(about = "Speed up or slow down a video — audio removed", long_about = None)]
struct Args {
    /// Input video file
    input: String,

    /// Output video file (optional — auto-named if omitted)
    #[arg(short, long)]
    output: Option<String>,

    /// Speed factor  (0.25 = quarter speed · 1.0 = normal · 4.0 = 4× faster)
    #[arg(short, long, default_value_t = 2.0)]
    speed: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "vidspeed=info".to_string()),
        )
        .with_target(false)
        .init();

    let args = Args::parse();

    if args.speed <= 0.0 || args.speed > 10.0 {
        eprintln!("Error: speed must be between 0.1 and 10.0, got {}", args.speed);
        std::process::exit(1);
    }

    cli::process_video(&args.input, args.output.as_deref(), args.speed).await?;

    Ok(())
}
