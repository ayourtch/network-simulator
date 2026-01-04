use network_simulator::config::SimulatorConfig;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from a file (default config.toml in project root)
    let cfg_str = fs::read_to_string("config.toml")?;
    let cfg: SimulatorConfig = toml::from_str(&cfg_str)?;
    // Run the simulator (this will build the fabric, compute routing, etc.)
    network_simulator::run(cfg).await?;
    Ok(())
}
