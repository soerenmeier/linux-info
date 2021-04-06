//!
//! A crate that should allow ease of use to get infos
//! about your linux system.  
//! Get information about your:
//! - cpu
//! - memory
//! - graphics card
//! - hard drive
//!
//! The api is not finished and feedback is appreciated.

/// Get cpu information.
pub mod cpu;
/// Get memory information.
pub mod memory;
// Get system information (uptime, hostname, usernames, groups).
pub mod system;

mod util;


pub mod size {
	use super::*;
	pub use util::Size;
}

// get cpu info
// get memory info
// get graphics info
// get process info
// get mdstats