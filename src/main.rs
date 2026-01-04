// src/main.rs

use std::process;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};
use network_simulator::config::SimulatorConfig;
use std::fs;

/// Simple CLI for the network simulator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the TOML configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Enable verbose (debug) logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Enable multipath routing
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Enable multipath routing")]
    multipath: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialise tracing subscriber – respects RUST_LOG and the -v flag.
    let filter = match args.verbose {
        0 => EnvFilter::new("network_simulator=info"),
        1 => EnvFilter::new("network_simulator=debug"),
        _ => EnvFilter::new("network_simulator=trace"),
    };
    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .init();

    let cfg_str = fs::read_to_string(&args.config)?;
    let mut cfg: SimulatorConfig = toml::from_str(&cfg_str)?;
    cfg.enable_multipath = args.multipath;
    if let Err(e) = network_simulator::run(cfg).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
    Ok(())
}
