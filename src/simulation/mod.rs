// src/simulation/mod.rs

use crate::topology::Link;
use tracing::debug;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};

// Global RNG protected by a Mutex. Initialized lazily.
static GLOBAL_RNG: once_cell::sync::Lazy<Mutex<Option<StdRng>>> = once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Initialize the global RNG with a seed. Call once during startup if a seed is provided.
pub fn init_rng(seed: u64) {
    let mut guard = GLOBAL_RNG.lock().unwrap();
    *guard = Some(StdRng::seed_from_u64(seed));
}

/// Obtain a mutable RNG. If the global RNG is set, use it; otherwise fall back to thread_rng.
fn get_rng() -> Box<dyn RngCore> {
    let mut guard = GLOBAL_RNG.lock().unwrap();
    if let Some(ref mut rng) = *guard {
        // Clone the StdRng to avoid holding the lock during usage.
        let cloned = rng.clone();
        Box::new(cloned)
    } else {
        Box::new(rand::thread_rng())
    }
}

/// Apply link characteristics (delay, jitter, loss) to a packet.
/// Returns `Ok(())` if the packet survives the link, or `Err` if it is dropped due to loss.
pub async fn simulate_link(link: &Link, packet: &[u8]) -> Result<(), &'static str> {
    // Increment packet counter for load‑balancing statistics
    use std::sync::atomic::Ordering;
    link.counter.fetch_add(1, Ordering::Relaxed);

    // MTU enforcement
    if let Some(mtu) = link.cfg.mtu {
        if packet.len() > mtu as usize {
            debug!("Packet size {} exceeds MTU {} on link {:?}", packet.len(), mtu, link.id);
            return Err("mtu_exceeded");
        }
    }

    // Simulate packet loss based on configured loss_percent (0.0 – 100.0).
    let mut rng = get_rng();
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

