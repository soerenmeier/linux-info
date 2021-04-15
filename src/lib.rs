//!
//! A crate that allows you to get information about your linux system.  
//! It's still a work in progress put the base modules *cpu*,
//! *memory*, *system* and *storage* are already there.  
//! Any feedback is welcome.
//!
//! ## Async
//! At the moment every method here reads from /proc/* which
//! does not benefit from async code.

/// Get cpu information.
pub mod cpu;
/// Get memory information.
pub mod memory;
// Get system information (uptime, hostname, usernames, groups).
pub mod system;
// Get storage information (partitions, mounts, stats, raids).
pub mod storage;

mod util;


pub mod unit {
	use super::*;
	pub use util::{DataSize, DataSizeUnit};
}