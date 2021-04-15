//!
//! A crate that should allow ease of use to get infos
//! about your linux system.  
//! Get information about your:
//! - cpu
//! - memory
//!
//! The api is not finished and feedback is appreciated.
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
// Get storage information (partitions, raids).
pub mod storage;

mod util;


pub mod unit {
	use super::*;
	pub use util::{DataSize, DataSizeUnit};
}

// get cpu info
// get memory info
// get graphics info
// get process info
// get mdstats