//! get information about drives and raids.  
//! Should this be called fs??

use crate::util::read_to_string_mut;
use crate::unit::DataSize;

use std::path::Path;
use std::{fs, io};
use std::convert::TryInto;

use byte_parser::{StrParser, ParseIterator, parse_iter};


/// Read partitions from /proc/partitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partitions {
	raw: String
}

impl Partitions {

	fn path() -> &'static Path {
		Path::new("/proc/partitions")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Read partitions from /proc/partitions.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
	}

	pub fn entries<'a>(&'a self) -> impl Iterator<Item=PartitionEntry<'a>> {
		self.raw.trim()
			.split('\n')
			.skip(2)// skip headers
			.map(PartitionEntry::from_str)
	}

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionEntry<'a> {
	raw: &'a str
}

impl<'a> PartitionEntry<'a> {

	fn from_str(raw: &'a str) -> Self {
		Self {raw}
	}

	/// returns every key and valu ein the cpu info
	pub fn values(&self) -> impl Iterator<Item=&'a str> {
		self.raw.split(' ')
			.map(str::trim)
			.filter(|s| !s.is_empty())
	}

	/// Returns the major value.
	pub fn major(&self) -> Option<usize> {
		self.values().nth(0)?
			.parse().ok()
	}

	/// Returns the minor value.
	pub fn minor(&self) -> Option<usize> {
		self.values().nth(1)?
			.parse().ok()
	}

	/// Returns the blocks value.
	pub fn blocks(&self) -> Option<usize> {
		self.values().nth(2)?
			.parse().ok()
	}

	/// Returns the name value.
	pub fn name(&self) -> Option<&'a str> {
		self.values().nth(3)
	}

}

/// Read mount points from /proc/self/mountinfo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountPoints {
	raw: String
}

impl MountPoints {

	fn path() -> &'static Path {
		Path::new("/proc/self/mountinfo")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Read mount points from /proc/self/mountinfo.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
	}

	/// Get the mount points.
	pub fn points<'a>(&'a self) -> impl Iterator<Item=MountPoint<'a>> {
		self.raw.trim()
			.split('\n')
			.map(MountPoint::from_str)
	}

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountPoint<'a> {
	raw: &'a str
}

impl<'a> MountPoint<'a> {

	fn from_str(raw: &'a str) -> Self {
		Self {raw}
	}

	/// Returns every value separated by a space.
	#[inline]
	pub fn values(&self) -> impl Iterator<Item=&'a str> {
		self.raw.split(' ')
	}

	/// A unique ID for the mount (may be reused after umount).
	pub fn mount_id(&self) -> Option<usize> {
		self.values().nth(0)?
			.parse().ok()
	}

	/// The ID of the parent mount (or of self for
	/// the root of this mount namespace's mount tree).
	pub fn parent_id(&self) -> Option<usize> {
		self.values().nth(1)?
			.parse().ok()
	}

	/// major:minor: the value of st_dev for files on this filesystem.
	#[inline]
	pub fn major_minor(&self) -> Option<&'a str> {
		self.values().nth(2)
	}

	/// Gets the major value.
	pub fn major(&self) -> Option<usize> {
		self.major_minor()?
			.split(':')
			.nth(0)?
			.parse().ok()
	}

	/// Gets the minor value.
	pub fn minor(&self) -> Option<usize> {
		self.major_minor()?
			.split(':')
			.nth(1)?
			.parse().ok()
	}

	/// the pathname of the directory in the filesystem
	/// which forms the root of this mount.
	pub fn root(&self) -> Option<&'a str> {
		self.values().nth(3)
	}

	/// The pathname of the mount point relative
	/// to the process's root directory.
	pub fn mount_point(&self) -> Option<&'a str> {
		self.values().nth(4)
	}

	/// Per-mount options.
	pub fn mount_options(&self) -> Option<&'a str> {
		self.values().nth(5)
	}

	/// Currently, the possible optional fields are `shared`, `master`,
	/// `propagate_from`, and `unbindable`.
	pub fn optional_fields(&self) -> impl Iterator<Item=(&'a str, Option<&'a str>)> {
		self.values().skip(6)
			.take_while(|&i| i != "-")
			.map(|opt| {
				let mut iters = opt.split(':');
				(
					iters.next().unwrap(),
					iters.next()
					// TODO: update when https://github.com/rust-lang/rust/issues/77998 gets closed
					// Some(iters.as_str()).filter(str::is_empty)
				)
			})
	}

	fn after_separator(&self) -> impl Iterator<Item=&'a str> {
		self.values().skip(5)
			.skip_while(|&i| i != "-")
			.skip(1)// skip separator
	}

	/// The filesystem type in the form "type[.subtype]".
	pub fn filesystem_type(&self) -> Option<&'a str> {
		// maybe parse subtype?
		self.after_separator().nth(0)
	}

	// Filesystem-specific information if available.  
	// Returns none if its the same as filesystem_type
	/// Filesystem-specific information.  
	/// df command uses this information as Filesystem.
	pub fn mount_source(&self) -> Option<&'a str> {
		self.after_separator().nth(1)
		// let src = self.after_separator().nth(1)?;
		// match self.filesystem_type() {
		// 	Some(fst) if fst == src => None,
		// 	_ => Some(src)
		// }
	}

	/// Per-superblock options.
	pub fn super_options(&self) -> Option<&'a str> {
		self.after_separator().nth(2)
	}

	/// Returns the filesystem statistics of this mount point.
	pub fn stats(&self) -> io::Result<FsStat> {
		FsStat::new(self.mount_point().unwrap_or(""))
	}

}

/// Filesystem statistics
#[derive(Clone)]
pub struct FsStat {
	raw: libc::statfs
}

impl FsStat {

	fn new(path: impl AsRef<Path>) -> io::Result<Self> {
		crate::util::statfs(path)
			.map(|raw| Self {raw})
	}

	/// Returns `true` if the total blocks is bigger than zero.
	pub fn has_blocks(&self) -> bool {
		self.total_blocks()
			.map(|b| b > 0)
			.unwrap_or(false)
	}

	/// The block size in bytes used for this filesystem.
	pub fn block_size(&self) -> Option<usize> {
		self.raw.f_bsize.try_into().ok()
	}

	/// The total block count.
	pub fn total_blocks(&self) -> Option<usize> {
		self.raw.f_blocks.try_into().ok()
	}

	/// The blocks that are still free may not all
	/// be accessible to unprivileged users.
	pub fn free_blocks(&self) -> Option<usize> {
		self.raw.f_bfree.try_into().ok()
	}

	/// The blocks that are free and accessible to unprivileged
	/// users.
	pub fn available_blocks(&self) -> Option<usize> {
		self.raw.f_bavail.try_into().ok()
	}

	/// The blocks that are already used.
	pub fn used_blocks(&self) -> Option<usize> {
		Some(self.total_blocks()? - self.free_blocks()?)
	}

	/// The size of the filesystem.
	pub fn total(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.total_blocks()? * self.block_size()?)
	}

	/// The size of the free space.
	pub fn free(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.free_blocks()? * self.block_size()?)
	}

	/// The size of the available space to unprivileged
	/// users.
	pub fn available(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.available_blocks()? * self.block_size()?)
	}

	/// The size of the space that is currently
	/// used.
	pub fn used(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.used_blocks()? * self.block_size()?)
	}

}

/// Read mount points from /proc/mdstat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raids {
	raw: String
}

impl Raids {

	fn path() -> &'static Path {
		Path::new("/proc/mdstat")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Read raid devices from /proc/mdstat.
	pub fn read() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Reloads information without allocating.
	pub fn reload(&mut self) -> io::Result<()> {
		read_to_string_mut(Self::path(), &mut self.raw)
	}

	/// Returns all listed devices in /proc/mdstat.
	pub fn raids(&self) -> impl Iterator<Item=Raid<'_>> {
		let mut first_line = false;
		parse_iter(
			StrParser::new(self.raw.trim()),
			move |parser| {
				if !first_line {
					parser.consume_while_byte_fn(|&b| b != b'\n');
					// remove newline
					parser.advance();
					first_line = true;
				}
				parser.peek()?;
				let key = parser.record()
					.while_byte_fn(|&b| b != b':')
					.consume_to_str()
					.trim();

				if key == "unused devices" {
					return None
				}
				// remove colon
				parser.advance();

				let mut parser = parser.record();
				let mut one = false;

				loop {

					if one && matches!(parser.peek(), Some(b'\n')) {
						// finished
						let s = parser.to_str().trim();
						parser.advance();
						return Some(Raid::from_str(key, s))
					}

					if one {
						one = false;
						continue
					}

					match parser.next() {
						Some(b'\n') => one = true,
						None => {
							// The end
							return Some(Raid::from_str(key, parser.to_str().trim()))
						},
						_ => {}
					}

				}
			}
		)
	}

}

// https://raid.wiki.kernel.org/index.php/Mdstat
/// A raid device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raid<'a> {
	name: &'a str,
	raw: &'a str
}

impl<'a> Raid<'a> {

	fn from_str(name: &'a str, raw: &'a str) -> Self {
		Self {name, raw}
	}

	/// Returns every line and their values with out the name.
	#[inline]
	pub fn values(&self) -> impl Iterator<Item=impl Iterator<Item=&'a str>> {
		self.raw.split('\n')
			.map(str::trim)
			.map(|l| l.split(' '))
	}

	/// The name of the raid for example `md0`.
	pub fn name(&self) -> &'a str {
		self.name
	}

	/// The state of the current device.
	pub fn state(&self) -> Option<&'a str> {
		self.values()
			.nth(0)?
			.nth(0)
	}

	fn line(&self, line: usize) -> impl Iterator<Item=&'a str> {
		let mut iter = self.values().nth(line);
		std::iter::from_fn(move || iter.as_mut()?.next())
	}

	/// Returns the kind of raid device.  
	/// Maybe in the future will return an enum.
	pub fn kind(&self) -> Option<&'a str> {
		self.line(0).nth(1)
	}

	/// Returns all devices (id, name) in this raid array.
	pub fn devices(&self) -> impl Iterator<Item=(usize, &'a str)> {
		self.line(0)
			.skip(2)
			.filter_map(|dev| {
				let mut split = dev.split(&['[', ']'][..]);
				let name = split.next()?;
				Some((
					split.next()?.parse().ok()?,
					name
				))
			})
	}

	/// Returns all usable blocks.
	pub fn usable_blocks(&self) -> Option<usize> {
		self.line(1)
			.nth(0)?
			.parse().ok()
	}

	/// The amount of devices that are currently used. Should
	/// be `raid.used_devices()? == raid.devices().count()`.
	pub fn used_devices(&self) -> Option<usize> {
		self.line(1)
			.find(|l| l.starts_with('['))?
			.split('/')
			.nth(0)?
			.strip_prefix('[')?
			.parse().ok()
	}

	/// The amount of devices that would be ideal for this
	/// array configuration.
	pub fn ideal_devices(&self) -> Option<usize> {
		self.line(1)
			.find(|l| l.starts_with('['))?
			.split('/')
			.nth(1)?
			.strip_suffix(']')?
			.parse().ok()
	}

	/// Returns the progress line if there is any, for example:  
	/// `[==>..................]  recovery = 12.6% (37043392/292945152) finish=127.5min speed=33440K/sec`
	pub fn progress(&self) -> Option<&'a str> {
		let l = self.raw.split('\n')
			.nth(2)?
			.trim();
		l.starts_with('[')
			.then(|| l)
	}

	/// Returns filesystem statistics to this raid array.
	pub fn stats(&self) -> io::Result<FsStat> {
		FsStat::new(format!("/dev/{}", self.name()))
	}

}


#[cfg(test)]
mod tests {
	use super::*;

	fn partitions() -> Partitions {
		Partitions::from_string("\
major minor  #blocks  name

   7        0     142152 loop0
   7        1     101528 loop1
 259        0  500107608 nvme0n1
 259        1     510976 nvme0n1p1\n\
		".into())
	}

	fn cmp_entry(major: usize, minor: usize, blocks: usize, name: &str, e: &PartitionEntry<'_>) {
		assert_eq!(e.major().unwrap(), major);
		assert_eq!(e.minor().unwrap(), minor);
		assert_eq!(e.blocks().unwrap(), blocks);
		assert_eq!(e.name().unwrap(), name);
	}

	#[test]
	fn all_partitions() {
		let part = partitions();
		let mut e = part.entries();
		println!("e: {:?}", part.entries().collect::<Vec<_>>());
		cmp_entry(7, 0, 142152, "loop0", &e.next().unwrap());
		cmp_entry(7, 1, 101528, "loop1", &e.next().unwrap());
		cmp_entry(259, 0, 500107608, "nvme0n1", &e.next().unwrap());
		cmp_entry(259, 1, 510976, "nvme0n1p1", &e.next().unwrap());
		assert!(e.next().is_none());
	}

	fn mount_points() -> MountPoints {
		MountPoints::from_string("\
26 29 0:5 / /dev rw,nosuid,noexec,relatime shared:2 - devtmpfs udev rw,size=8123832k,nr_inodes=2030958,mode=755
27 26 0:24 / /dev/pts rw,nosuid,noexec,relatime shared:3 - devpts devpts rw,gid=5,mode=620,ptmxmode=000
35 33 0:30 / /sys/fs/cgroup/systemd rw,nosuid,nodev,noexec,relatime shared:11 other - cgroup cgroup rw,xattr,name=systemd
2509 28 0:25 /snapd/ns /run/snapd/ns rw,nosuid,nodev,noexec,relatime - tmpfs tmpfs rw,size=1631264k,mode=755
2893 2509 0:4 mnt:[4026532961] /run/snapd/ns/snap-store.mnt rw - nsfs nsfs rw\n\
		".into())
	}

	fn cmp_point(
		mount_id: usize,
		parent_id: usize,
		major_minor: &str,
		root: &str,
		mount_point: &str,
		mount_options: &str,
		optional_fields: &[(&str, Option<&str>)],
		filesystem_type: &str,
		mount_source: &str,
		super_options: &str,
		point: &MountPoint<'_>
	) {
		assert_eq!(point.mount_id().unwrap(), mount_id);
		assert_eq!(point.parent_id().unwrap(), parent_id);
		assert_eq!(point.major_minor().unwrap(), major_minor);
		assert_eq!(point.root().unwrap(), root);
		assert_eq!(point.mount_point().unwrap(), mount_point);
		assert_eq!(point.mount_options().unwrap(), mount_options);
		assert_eq!(point.optional_fields().collect::<Vec<_>>(), optional_fields);
		assert_eq!(point.filesystem_type().unwrap(), filesystem_type);
		assert_eq!(point.mount_source().unwrap(), mount_source);
		assert_eq!(point.super_options().unwrap(), super_options);
	}

	#[test]
	fn all_mount_points() {
		let mt = mount_points();
		let mut mt = mt.points();
		cmp_point(
			26, 29, "0:5", "/", "/dev", "rw,nosuid,noexec,relatime",
			&[("shared", Some("2"))], "devtmpfs", "udev",
			"rw,size=8123832k,nr_inodes=2030958,mode=755",
			&mt.next().unwrap()
		);
		cmp_point(
			27, 26, "0:24", "/", "/dev/pts", "rw,nosuid,noexec,relatime",
			&[("shared", Some("3"))], "devpts", "devpts",
			"rw,gid=5,mode=620,ptmxmode=000",
			&mt.next().unwrap()
		);
		cmp_point(
			35, 33, "0:30", "/", "/sys/fs/cgroup/systemd", "rw,nosuid,nodev,noexec,relatime",
			&[("shared", Some("11")), ("other", None)], "cgroup", "cgroup", "rw,xattr,name=systemd",
			&mt.next().unwrap()
		);
		cmp_point(
			2509, 28, "0:25", "/snapd/ns", "/run/snapd/ns", "rw,nosuid,nodev,noexec,relatime",
			&[], "tmpfs", "tmpfs", "rw,size=1631264k,mode=755",
			&mt.next().unwrap()
		);
		cmp_point(
			2893, 2509, "0:4", "mnt:[4026532961]", "/run/snapd/ns/snap-store.mnt", "rw",
			&[], "nsfs", "nsfs", "rw",
			&mt.next().unwrap()
		);
	}

	#[test]
	fn raid_case_1() {
		let raids = Raids::from_string("\
Personalities : [raid1] [linear] [multipath] [raid0] [raid6] [raid5] [raid4] [raid10] 
md10 : active raid1 sdd[0] sdc[1]
      3906886464 blocks super 1.2 [2/2] [UU]
      bitmap: 0/30 pages [0KB], 65536KB chunk

md0 : active raid1 sdb[1] sda[0]
      499975488 blocks super 1.2 [2/2] [UU]
      bitmap: 3/4 pages [12KB], 65536KB chunk

unused devices: <none>\n".into());
		assert_eq!(raids.raids().count(), 2);
		let first = raids.raids().next().unwrap();
		assert_eq!(first.name(), "md10");
		assert_eq!(first.used_devices().unwrap(), 2);
		assert_eq!(first.ideal_devices().unwrap(), 2);
		assert!(first.progress().is_none());
		assert_eq!(first.devices().count(), first.used_devices().unwrap());
	}

	#[test]
	fn raid_case_2() {
		let raids = Raids::from_string("\
Personalities : [raid1] [raid6] [raid5] [raid4]
md127 : active raid5 sdh1[6] sdg1[4] sdf1[3] sde1[2] sdd1[1] sdc1[0]
      1464725760 blocks level 5, 64k chunk, algorithm 2 [6/5] [UUUUU_]
      [==>..................]  recovery = 12.6% (37043392/292945152) finish=127.5min speed=33440K/sec

unused devices: <none>\n".into());
		assert_eq!(raids.raids().count(), 1);
		let first = raids.raids().next().unwrap();
		let comp_dev: Vec<_> = first.devices().collect();
		assert_eq!(comp_dev, [(6, "sdh1"), (4, "sdg1"), (3, "sdf1"), (2, "sde1"), (1, "sdd1"), (0, "sdc1")]);
		assert_eq!(first.kind().unwrap(), "raid5");
		assert_eq!(first.usable_blocks().unwrap(), 1464725760);
		assert_eq!(first.used_devices().unwrap(), 6);
		assert_eq!(first.ideal_devices().unwrap(), 5);
		assert_eq!(first.progress().unwrap(), "[==>..................]  recovery = 12.6% (37043392/292945152) finish=127.5min speed=33440K/sec");
		assert_eq!(first.devices().count(), first.used_devices().unwrap());
	}

	#[test]
	fn raid_case_3() {
		let raids = Raids::from_string("\
Personalities : [linear] [raid0] [raid1] [raid5] [raid4] [raid6]
md0 : active raid6 sdf1[0] sde1[1] sdd1[2] sdc1[3] sdb1[4] sda1[5] hdb1[6]
      1225557760 blocks level 6, 256k chunk, algorithm 2 [7/7] [UUUUUUU]
      bitmap: 0/234 pages [0KB], 512KB chunk

unused devices: <none>\n".into());
		assert_eq!(raids.raids().count(), 1);
		let first = raids.raids().next().unwrap();
		assert_eq!(first.devices().count(), first.used_devices().unwrap());
	}

}

// get block number
// /sys/block/<part>/dev   returns 7:0
// uuid /sys/dev/block/7:0/dm/uuid

/*
Personalities : [raid1] [linear] [multipath] [raid0] [raid6] [raid5] [raid4] [raid10] 
md10 : active raid1 sdd[0] sdc[1]
      3906886464 blocks super 1.2 [2/2] [UU]
      bitmap: 0/30 pages [0KB], 65536KB chunk

md0 : active raid1 sdb[1] sda[0]
      499975488 blocks super 1.2 [2/2] [UU]
      bitmap: 3/4 pages [12KB], 65536KB chunk

unused devices: <none>


Personalities : [raid1] [raid6] [raid5] [raid4]
md127 : active raid5 sdh1[6] sdg1[4] sdf1[3] sde1[2] sdd1[1] sdc1[0]
      1464725760 blocks level 5, 64k chunk, algorithm 2 [6/5] [UUUUU_]
      [==>..................]  recovery = 12.6% (37043392/292945152) finish=127.5min speed=33440K/sec

unused devices: <none> 


Personalities : [linear] [raid0] [raid1] [raid5] [raid4] [raid6]
md0 : active raid6 sdf1[0] sde1[1] sdd1[2] sdc1[3] sdb1[4] sda1[5] hdb1[6]
      1225557760 blocks level 6, 256k chunk, algorithm 2 [7/7] [UUUUUUU]
      bitmap: 0/234 pages [0KB], 512KB chunk

unused devices: <none>
*/