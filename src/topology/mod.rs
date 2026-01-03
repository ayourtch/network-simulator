// src/topology/mod.rs

pub mod router;
pub mod link;
pub mod fabric;

pub use router::{Router, RouterId, RouterStats};
pub use link::{Link, LinkId, LinkConfig};
pub use fabric::Fabric;
