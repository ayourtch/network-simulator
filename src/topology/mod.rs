// src/topology/mod.rs

pub mod fabric;
pub mod link;
pub mod router;

pub use fabric::Fabric;
pub use link::{Link, LinkConfig, LinkId};
pub use router::{Router, RouterId, RouterStats};
