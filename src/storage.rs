//! get information about drives and raids.  
//! Should this be called fs??

use crate::util::read_to_string_mut;
use crate::unit::DataSize;

use std::path::Path;
use std::{fs, io};
use std::convert::TryInto;


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

	pub fn is_empty(&self) -> bool {
		self.total_blocks()
			.map(|t| t == 0)
			.unwrap_or(false)
	}

	// in bytes
	pub fn block_size(&self) -> Option<usize> {
		self.raw.f_bsize.try_into().ok()
	}

	pub fn total_blocks(&self) -> Option<usize> {
		self.raw.f_blocks.try_into().ok()
	}

	pub fn free_blocks(&self) -> Option<usize> {
		self.raw.f_bfree.try_into().ok()
	}

	pub fn available_blocks(&self) -> Option<usize> {
		self.raw.f_bavail.try_into().ok()
	}

	/// total - available
	pub fn used_blocks(&self) -> Option<usize> {
		Some(self.total_blocks()? - self.free_blocks()?)
	}

	pub fn total(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.total_blocks()? * self.block_size()?)
	}

	pub fn free(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.free_blocks()? * self.block_size()?)
	}

	pub fn available(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.available_blocks()? * self.block_size()?)
	}

	pub fn used(&self) -> Option<DataSize> {
		DataSize::from_size_bytes(self.used_blocks()? * self.block_size()?)
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

}

// get block number
// /sys/block/<part>/dev   returns 7:0
// uuid /sys/dev/block/7:0/dm/uuid