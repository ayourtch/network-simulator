// src/simulation/mod.rs

use crate::topology::{Link, LinkConfig};
use tracing::debug;
use rand::Rng;
use tokio::time::{sleep, Duration};

/// Apply link characteristics (delay, jitter, loss) to a packet.
/// Returns `Ok(())` if the packet survives the link, or `Err` if it is dropped due to loss.
pub async fn simulate_link(link: &Link, _packet: &[u8]) -> Result<(), &'static str> {
    // Simulate packet loss based on configured loss_percent (0.0 – 100.0).
    let mut rng = rand::thread_rng();
    if rng.gen_range(0.0..100.0) < link.cfg.loss_percent as f64 {
        debug!("Packet dropped on link {:?} due to loss ({}%)", link.id, link.cfg.loss_percent);
        return Err("packet lost");
    }

    // Compute total delay = base delay + jitter (random 0..=jitter).
    let jitter = if link.cfg.jitter_ms > 0 {
        rng.gen_range(0..=link.cfg.jitter_ms)
    } else {
        0
    };
    let total_delay = link.cfg.delay_ms + jitter;
    if total_delay > 0 {
        debug!("Delaying packet on link {:?} by {} ms (jitter {} ms)", link.id, link.cfg.delay_ms, jitter);
        sleep(Duration::from_millis(total_delay as u64)).await;
    }
    debug!("Packet passed through link {:?}", link.id);
    Ok(())
}

