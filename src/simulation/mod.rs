// src/simulation/mod.rs

use crate::topology::Link;
use tracing::debug;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};
use thiserror::Error;

// Global RNG protected by a Mutex. Initialized with entropy, can be reseeded via init_rng.
static GLOBAL_RNG: Lazy<Mutex<StdRng>> = Lazy::new(|| Mutex::new(StdRng::from_entropy()));

/// Initialize the global RNG with a seed. Call once during startup if a seed is provided.
pub fn init_rng(seed: u64) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    *rng = StdRng::seed_from_u64(seed);
}

/// Errors that can arise during link simulation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SimulationError {
    #[error("Packet lost due to link loss simulation")]
    PacketLost,
    #[error("Packet size {packet_size} exceeds MTU {mtu}")]
    MtuExceeded { packet_size: usize, mtu: u32 },
    #[error("Other simulation error: {0}")]
    Other(String),
}

/// Apply link characteristics (delay, jitter, loss) to a packet.
/// Returns `Ok(())` if the packet survives the link, or `Err` if it is dropped due to loss or other issues.
pub async fn simulate_link(link: &Link, packet: &[u8]) -> Result<(), SimulationError> {
    // Increment packet counter for load‑balancing statistics
    use std::sync::atomic::Ordering;
    link.counter.fetch_add(1, Ordering::Relaxed);

    // MTU enforcement
    if let Some(mtu) = link.cfg.mtu {
        if packet.len() > mtu as usize {
            debug!("Packet size {} exceeds MTU {} on link {:?}", packet.len(), mtu, link.id);
            return Err(SimulationError::MtuExceeded { packet_size: packet.len(), mtu });
        }
    }

    // Simulate packet loss and compute jitter without holding the global RNG lock across await points.
    let (loss_occurred, jitter_val) = {
        let mut rng = GLOBAL_RNG.lock().unwrap();
        let loss = rng.gen_range(0.0..100.0) < link.cfg.loss_percent as f64;
        let jitter = if link.cfg.jitter_ms > 0 {
            // Generate jitter in the range [-jitter_ms, +jitter_ms]
            let range = -(link.cfg.jitter_ms as i32)..=link.cfg.jitter_ms as i32;
            rng.gen_range(range) // returns i32
        } else {
            0
        };
        (loss, jitter)
    };
    if loss_occurred {
        debug!("Packet dropped on link {:?} due to loss ({}%)", link.id, link.cfg.loss_percent);
        return Err(SimulationError::PacketLost);
    }

    // Compute total delay = base delay + jitter (can be negative).
    let jitter = jitter_val;
    // Ensure total delay is non‑negative
    let total_delay_i32 = link.cfg.delay_ms as i32 + jitter;
    let total_delay = if total_delay_i32 < 0 { 0 } else { total_delay_i32 as u32 };
    if total_delay > 0 {
        debug!("Delaying packet on link {:?} by {} ms (jitter {} ms)", link.id, link.cfg.delay_ms, jitter);
        sleep(Duration::from_millis(total_delay as u64)).await;
    }
    debug!("Packet passed through link {:?}", link.id);
    Ok(())
}

