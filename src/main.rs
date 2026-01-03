// src/main.rs

use std::process;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

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
}

#[tokio::main]
async fn main() {
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

    if let Err(e) = network_simulator::run(&args.config).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
