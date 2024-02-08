//! get system information (uptime, hostname, os release, load average, usernames, groups).

use crate::util::read_to_string_mut;

use std::{fs, io};
use std::path::Path;
use std::time::Duration;
use std::ops::Sub;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuStat {
	/// user: normal processes executing in user mode
	pub user: usize,
	/// nice: niced processes executing in user mode
	pub nice: usize,
	/// system: processes executing in kernel mode
	pub system: usize,
	/// idle: twiddling thumbs
	pub idle: usize,
	/// iowait: waiting for I/O to complete
	pub iowait: usize,
	/// irq: servicing interrupts
	pub irq: usize,
	/// softirq: servicing softirqs
	pub softirq: usize
}

impl CpuStat {
	// Calculate total time
	pub fn total_time(&self) -> usize {
		self.user + self.nice + self.system + self.idle + self.iowait +
		self.irq + self.softirq
	}

	// Calculate total active time (excluding idle and iowait)
	pub fn active_time(&self) -> usize {
		self.user + self.nice + self.system + self.irq + self.softirq
	}

	// Calculate CPU usage 0-1
	//
	// previous needs to be older
	pub fn usage(&self, previous: &Self) -> f64 {
		let diff = *self - *previous;

		if diff.total_time() == 0 {
			return 0.0;
		}

		diff.active_time() as f64 / diff.total_time() as f64
	}
}

impl Sub for CpuStat {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self {
			user: self.user - other.user,
			nice: self.nice - other.nice,
			system: self.system - other.system,
			idle: self.idle - other.idle,
			iowait: self.iowait - other.iowait,
			irq: self.irq - other.irq,
			softirq: self.softirq - other.softirq,
		}
	}
}

impl FromIterator<usize> for CpuStat {
	fn from_iter<T>(iter: T) -> Self
	where T: IntoIterator<Item = usize> {
		let mut iter = iter.into_iter();

		Self {
			user: iter.next().unwrap_or(0),
			nice: iter.next().unwrap_or(0),
			system: iter.next().unwrap_or(0),
			idle: iter.next().unwrap_or(0),
			iowait: iter.next().unwrap_or(0),
			irq: iter.next().unwrap_or(0),
			softirq: iter.next().unwrap_or(0)
		}
	}
}

/// Read the load average from /proc/loadavg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
	raw: String
}

impl Stat {
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
	pub fn values<'a>(&'a self) -> impl Iterator<Item=(
		&'a str,
		impl Iterator<Item=usize> + '_
	)> {
		self.raw.trim().lines()
			.map(str::trim)
			.filter_map(|s| {
				let (key, rest) = s.split_once(' ')?;

				Some((key, rest.split(' ').filter_map(|v| v.parse().ok())))
			})
	}

	pub fn cpu(&self) -> Option<CpuStat> {
		self.values().find(|(k, _)| *k == "cpu")
			.map(|(_, v)| v.collect())
	}

	pub fn cpu_nth(&self, nth: usize) -> Option<CpuStat> {
		let nk = format!("cpu{}", nth);
		self.values().find(|(k, _)| *k == nk)
			.map(|(_, v)| v.collect())
	}
}


// TODO add https://www.idnt.net/en-US/kb/941772
// /proc/stat


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

	#[test]
	fn stat() {
		let first = Stat::from_string("\
cpu  47500 2396 21138 741776 6759 0 516 0 0 0
cpu0 1657 25 649 31631 152 0 40 0 0 0
cpu1 1895 140 624 31335 197 0 9 0 0 0
cpu2 2155 69 696 31185 101 0 2 0 0 0
cpu3 2830 72 723 30280 259 0 15 0 0 0
cpu4 2378 11 776 30813 247 0 1 0 0 0
cpu5 2402 326 724 30541 193 0 0 0 0 0
cpu6 1488 13 1217 31159 76 0 1 0 0 0
cpu7 1537 50 861 31563 111 0 12 0 0 0
cpu8 2164 22 1279 30611 120 0 0 0 0 0
cpu9 2760 24 682 30418 292 0 6 0 0 0
cpu10 2454 440 676 30409 206 0 0 0 0 0
cpu11 1944 10 709 31251 284 0 0 0 0 0
cpu12 2050 75 957 30479 634 0 0 0 0 0
cpu13 1751 180 583 31385 303 0 6 0 0 0
cpu14 1684 77 753 30998 414 0 162 0 0 0
cpu15 1922 53 561 31603 73 0 0 0 0 0
cpu16 2189 75 1108 30151 605 0 36 0 0 0
cpu17 2113 240 1212 30252 393 0 0 0 0 0
cpu18 1547 89 1132 30984 346 0 68 0 0 0
cpu19 2009 87 1479 30265 360 0 7 0 0 0
cpu20 1832 20 1260 30762 268 0 1 0 0 0
cpu21 1396 10 669 31952 157 0 0 0 0 0
cpu22 1466 249 908 30772 567 0 142 0 0 0
cpu23 1868 33 890 30967 388 0 0 0 0 0
intr 5968724 39 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1830 0 21 658 0 0 0 0 0 0 0 185 0 206 0 0 0 0 0 0 0 0 0 0 0 0 0 10756 12407 32939 620 3309 8687 22003 1735 1 0 0 0 0 0 0 0 492 0 0 0 0 0 0 0 0 0 0 0 0 23 67418 0 169 169 169 169 0 4770 0 0 0 0 0 0 0 72534 90229 23 36684 79 45360 4 74224 17 64117 72 65789 38 87 25 212 0 0 0 2973 0 3527 0 82311 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
ctxt 9220606
btime 1698004999
processes 10505
procs_running 3
procs_blocked 1
softirq 1572362 6570 73617 6 106501 103799 0 729 724985 18 556137\n\
		".into());

		assert!(first.cpu_nth(0).is_some());
		assert_eq!(first.cpu().unwrap(), CpuStat {
			user: 47500,
			nice: 2396,
			system: 21138,
			idle: 741776,
			iowait: 6759,
			irq: 0,
			softirq: 516
		});


		let second = Stat::from_string("\
cpu  598326 3695 207316 16449301 11326 0 5035 0 0 0
cpu0 17756 59 5304 695144 394 0 2671 0 0 0
cpu1 24815 195 5214 689481 343 0 281 0 0 0
cpu2 23030 111 5271 691609 188 0 28 0 0 0
cpu3 37215 147 7633 674968 428 0 23 0 0 0
cpu4 35260 43 6956 677812 425 0 2 0 0 0
cpu5 32865 364 7053 679702 371 0 25 0 0 0
cpu6 15264 65 17016 681953 264 0 2 0 0 0
cpu7 25513 94 15448 677409 368 0 30 0 0 0
cpu8 23536 72 16582 678224 276 0 0 0 0 0
cpu9 27646 68 5548 685186 406 0 1031 0 0 0
cpu10 27508 495 5536 686719 309 0 0 0 0 0
cpu11 25780 38 5424 688852 413 0 0 0 0 0
cpu12 27720 133 5704 686025 849 0 0 0 0 0
cpu13 25348 288 5167 689086 472 0 10 0 0 0
cpu14 22885 160 5622 690560 608 0 287 0 0 0
cpu15 25662 95 6380 688143 248 0 0 0 0 0
cpu16 24917 118 7501 686875 852 0 106 0 0 0
cpu17 24053 320 7208 688030 711 0 0 0 0 0
cpu18 19499 154 16800 681362 768 0 128 0 0 0
cpu19 21094 126 16548 680076 501 0 12 0 0 0
cpu20 21863 58 17398 678483 597 0 14 0 0 0
cpu21 23657 93 5421 691105 246 0 15 0 0 0
cpu22 22550 310 5334 690909 719 0 358 0 0 0
cpu23 22883 81 5240 691578 558 0 2 0 0 0
intr 93982176 39 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 47273 0 21 658 0 0 0 0 0 0 0 462 0 520 0 0 0 0 0 0 0 0 0 0 0 0 0 67446 34716 59371 10575 28561 29891 81562 29376 1 0 0 0 0 0 0 0 718 0 0 0 0 0 0 0 0 0 0 0 0 23 94910 0 3608 3608 3608 3608 0 82604 0 0 0 0 0 0 0 127375 128843 23 60563 802 75923 571 96606 158 104005 128 94386 71 214 204 401 0 0 0 2973 0 5386 0 1757983 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
ctxt 157231394
btime 1698004999
processes 93053
procs_running 5
procs_blocked 0
softirq 19512683 120053 1138489 8 420631 143436 0 10350 10473743 18 7205955\n\
		".into());

		let first_cpu = first.cpu().unwrap();
		let second_cpu = second.cpu().unwrap();

		let usage = second_cpu.usage(&first_cpu);
		assert_eq!(usage, 0.04514286735257322);
	}
}