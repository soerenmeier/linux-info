//! get system information (uptime, hostname, usernames, groups).

use std::{fs, io};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uptime {
	raw: String
}

impl Uptime {

	fn path() -> &'static Path {
		Path::new("/proc/uptime")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Reads uptime from /proc/uptime.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Main method to get uptime values. Returns every entry.
	pub fn all_infos<'a>(&'a self) -> impl Iterator<Item=Duration> + 'a {
		self.raw.split(' ')
			.filter_map(|v| v.trim().parse().ok())
			.map(Duration::from_secs_f64)
	}

	/// Get the system uptime.
	pub fn uptime(&self) -> Option<Duration> {
		self.all_infos().next()
	}

	/// Get the sum of how much time each core has spent idle.  
	/// Should be idletime / cores to get the real idle time.
	pub fn idletime(&self) -> Option<Duration> {
		self.all_infos().nth(1)
	}

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hostname {
	raw: String
}

impl Hostname {

	fn path() -> &'static Path {
		Path::new("/proc/sys/kernel/hostname")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Reads hostname from /proc/sys/kernel/hostname.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Get hostname as str.
	pub fn hostname(&self) -> &str {
		self.raw.trim()
	}

	/// Get hostname as raw String (may contain whitespace).
	pub fn into_string(self) -> String {
		self.raw
	}

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsRelease {
	raw: String
}

impl OsRelease {

	fn path() -> &'static Path {
		Path::new("/proc/sys/kernel/osrelease")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Reads hostname from /proc/sys/kernel/osrelease.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Get os release as str.
	pub fn full_str(&self) -> &str {
		self.raw.trim()
	}

	/// Get os release as raw String (may contain whitespace).
	pub fn into_string(self) -> String {
		self.raw
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	fn uptime() -> Uptime {
		Uptime::from_string("220420.83 5275548.45\n".into())
	}

	#[test]
	fn uptime_methods() {
		// uptime
		assert_eq!(uptime().uptime().unwrap().as_secs(), 220420);
		// idle time
		assert_eq!(uptime().idletime().unwrap().as_secs(), 5275548);
	}

	#[test]
	fn hostname() {
		// a useless test
		let name = Hostname::from_string("test-hostname\n".into());
		assert_eq!(name.hostname(), "test-hostname");
	}

	#[test]
	fn os_release() {
		// a useless test
		let name = OsRelease::from_string("test-hostname\n".into());
		assert_eq!(name.full_str(), "test-hostname");
	}
}
