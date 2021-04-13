//! get system information (uptime, hostname, os release, load average, usernames, groups).

use crate::util::read_to_string_mut;

use std::{fs, io};
use std::path::Path;
use std::time::Duration;

/// Read uptime information from /proc/uptime.
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

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
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

/// Read the hostname from /proc/sys/kernel/hostname.
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

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
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

/// Read the hostname from /proc/sys/kernel/osrelease.
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

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
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

/// Read the load average from /proc/loadavg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadAvg {
	raw: String
}

impl LoadAvg {

	fn path() -> &'static Path {
		Path::new("/proc/loadavg")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Read load average from /proc/loadavg.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
	}

	/// Get all key and values.
	pub fn values<'a>(&'a self) -> impl Iterator<Item=&'a str> {
		self.raw.split(' ')
			.map(str::trim)
	}

	/// Get the average of jobs in the queue or waiting for disk I/O.  
	/// The values are averaged over (1 min, 5 min, 15 min).
	pub fn average(&self) -> Option<(f32, f32, f32)> {
		let mut vals = self.values()
			.take(3)
			.map(|v| v.parse().ok());
		Some((vals.next()??, vals.next()??, vals.next()??))
	}

	/// Returns two values (runnable threads, running threads).
	pub fn threads(&self) -> Option<(usize, usize)> {
		let mut vals = self.values()
			.nth(3)?
			.split('/')
			.map(|v| v.parse().ok());
		Some((vals.next()??, vals.next()??))
	}

	/// Returns the PID of the most recent process.
	pub fn newest_pid(&self) -> Option<u32> {
		self.values().last()?
			.parse().ok()
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

	#[test]
	fn load_avg() {
		let s = LoadAvg::from_string("13.37 15.82 16.64 14/1444 436826\n".into());
		assert_eq!(s.average().unwrap(), (13.37, 15.82, 16.64));
		assert_eq!(s.threads().unwrap(), (14, 1444));
		assert_eq!(s.newest_pid().unwrap(), 436826);
	}
}
