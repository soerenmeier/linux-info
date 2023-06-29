#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

/// Get cpu information.
pub mod cpu;
/// Get memory information.
pub mod memory;
// Get system information (uptime, hostname, usernames, groups).
pub mod system;
// Get storage information (partitions, mounts, stats, raids).
pub mod storage;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
/// get bios / system information
pub mod bios;
#[cfg(feature = "network")]
#[cfg_attr(docsrs, doc(cfg(feature = "network")))]
pub mod network;

mod util;


pub mod unit {
	use super::*;
	pub use util::{DataSize, DataSizeUnit};
}