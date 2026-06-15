mod cli;
mod config;
mod processor;
mod web;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "vidspeed")]
#[command(about = "Speed up videos without sound - CLI and Web Server", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run as web server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Host to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Process a single video file
    Process {
        /// Input video file
        input: String,

        /// Output video file (optional)
        #[arg(short, long)]
        output: Option<String>,

        /// Speed factor (1.0 = normal, >1 faster, <1 slower)
        #[arg(short, long, default_value_t = 2.0)]
        speed: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "vidspeed=info".to_string()))
        .with_target(false)
        .with_thread_ids(true)
        .init();

    let args = Args::parse();

    match args.command {
        Some(Commands::Server { port, host }) => {
            let config = config::Config::from_env()?;
            web::run_server(config, port, host).await?;
        }
        Some(Commands::Process {
            input,
            output,
            speed,
        }) => {
            cli::process_video(&input, output.as_deref(), speed).await?;
        }
        None => {
            println!("Usage: vidspeed <COMMAND>");
            println!("  vidspeed process -i <file> -s <speed>");
            println!("  vidspeed server");
        }
    }

    Ok(())
}
