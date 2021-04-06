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

	/// Load uptime synchronously.
	pub fn load_sync() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Load uptime asynchronously.
	#[cfg(feature = "async")]
	pub async fn load_async() -> io::Result<Self> {
		Ok(Self {
			raw: tokio::fs::read_to_string(Self::path()).await?
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
}
