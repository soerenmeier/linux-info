//!
//! The data is retrieved from `/proc/meminfo`
//!
//! To list all availabe key's [linuxwiki.org](https://linuxwiki.org/proc/meminfo). Or you can use the api
//! ```
//! use linux_info::memory::Memory;
//! let info = Memory::read().unwrap();
//! let keys = info.keys();
//! ```

use crate::unit::DataSize;
use crate::util::read_to_string_mut;

use std::path::Path;
use std::{fs, io};

/// Read memory information from /proc/meminfo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory {
	raw: String
}

impl Memory {

	fn path() -> &'static Path {
		Path::new("/proc/meminfo")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Read memory infos from /proc/meminfo.
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
	pub fn values<'a>(&'a self) -> impl Iterator<Item=(&'a str, &'a str)> {
		self.raw.split('\n')
			.filter_map(|line| {
				// TODO: after 1.52 update tot split_once
				let mut iter = line.splitn(2, ':');
				let (key, value) = (iter.next()?, iter.next()?);
				Some((key.trim(), value.trim()))
			})
	}

	/// get all keys.
	pub fn keys<'a>(&'a self) -> impl Iterator<Item=&'a str> {
		self.values()
			.map(|(k, _)| k)
	}

	/// Get value by key.
	pub fn value<'a>(&'a self, key: &str) -> Option<&'a str> {
		self.values()
			.find_map(|(k, v)| k.eq_ignore_ascii_case(key).then(|| v))
	}

	/// Get size by key.
	pub fn size_value<'a>(&'a self, key: &str) -> Option<DataSize> {
		self.value(key)
			.and_then(DataSize::from_str)
	}

	/// Returns the total memory.
	pub fn total_memory(&self) -> Option<DataSize> {
		self.size_value("MemTotal")
	}

	/// Returns the free memory.
	pub fn free_memory(&self) -> Option<DataSize> {
		self.size_value("MemFree")
	}

	/// Returns the available memory.
	pub fn available_memory(&self) -> Option<DataSize> {
		self.size_value("MemAvailable")
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::unit::DataSizeUnit;

	fn memory_info() -> Memory {
		Memory::from_string("\
MemTotal:       32853280 kB
MemFree:          919776 kB
MemAvailable:   28781828 kB
Buffers:          298460 kB
Cached:         27104800 kB
SwapCached:          168 kB
Active:          7764012 kB
Inactive:       22289624 kB
Active(anon):    2257064 kB
Inactive(anon):   624500 kB
Active(file):    5506948 kB
Inactive(file): 21665124 kB
Unevictable:          16 kB
Mlocked:              16 kB
SwapTotal:       2097148 kB
SwapFree:        2094844 kB
Dirty:               360 kB
Writeback:             0 kB
AnonPages:       2650504 kB
Mapped:           760008 kB
Shmem:            231188 kB
KReclaimable:    1154740 kB
Slab:            1529684 kB
SReclaimable:    1154740 kB
SUnreclaim:       374944 kB
KernelStack:       21600 kB
PageTables:        31948 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:    18523788 kB
Committed_AS:    9191380 kB
VmallocTotal:   34359738367 kB
VmallocUsed:       62668 kB
VmallocChunk:          0 kB
Percpu:            37376 kB
HardwareCorrupted:     0 kB
AnonHugePages:         0 kB
ShmemHugePages:        0 kB
ShmemPmdMapped:        0 kB
FileHugePages:         0 kB
FilePmdMapped:         0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
DirectMap4k:      922564 kB
DirectMap2M:    10518528 kB
DirectMap1G:    22020096 kB\
		".into())
	}

	// #[test]
	// fn info_to_vec() {
	// 	let cpu_info = cpu_info();
	// 	let v: Vec<_> = cpu_info.all_infos().collect();
	// 	assert_eq!(v.len(), 2);
	// }

	// #[test]
	// fn info_values() {
	// 	let info = cpu_info();
	// 	let mut values = info.all_infos();
	// 	let first = values.next().unwrap();
	// 	println!("first {:?}", first.values().collect::<Vec<_>>());
	// 	let model_name = first.value("model name").unwrap();
	// 	assert_eq!(model_name, "AMD Ryzen 9 3900XT 12-Core Processor");
	// }

	// #[test]
	// fn count_cores() {
	// 	let cpu_info = cpu_info();
	// 	assert_eq!(cpu_info.cores(), 2);
	// }

	// #[test]
	// fn unique_values() {
	// 	let cpu_info = cpu_info();
	// 	let un = cpu_info.unique_values("model name");
	// 	assert_eq!(un.len(), 1);
	// }

	#[test]
	fn total_memory() {
		let mem_info = memory_info();
		let total_memory = mem_info.total_memory().unwrap();
		assert_eq!(total_memory.to(&DataSizeUnit::Kb), 32853280.0);
	}

}