//!
//! See example `dmidecode_mini` on how to use this.
//!
//! ## Support
//! only SMBIOS 3.0+ is supported.
//!
//! To be able to use this the following files need to exist
//! `/sys/firmware/dmi/tables/{smbios_entry_point, DMI}` and you need permission
//! to read them.


mod low_level;

use std::io;

pub use uuid::Uuid;

use low_level::{
	EntryPoint, Structures, StructureKind, BiosInformation, SystemInformation
};

#[derive(Debug, PartialEq, Eq)]
pub struct Bios {
	entry_point: EntryPoint,
	structures: Structures
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BiosInfo<'a> {
	pub vendor: &'a str,
	pub version: &'a str,
	pub release_date: &'a str,
	pub major: u8,
	pub minor: u8
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SystemInfo<'a> {
	pub manufacturer: &'a str,
	pub product_name: &'a str,
	pub version: &'a str,
	pub serial_number: &'a str,
	/// is exactly 16bytes long
	pub uuid: Uuid,
	pub sku_number: &'a str,
	pub family: &'a str
}

impl Bios {
	pub fn read() -> io::Result<Self> {
		let entry_point = EntryPoint::read()?;
		Ok(Self {
			structures: Structures::read(entry_point.table_max)?,
			entry_point
		})
	}

	pub fn bios_info(&self) -> Option<BiosInfo> {
		let stru = self.structures.structures()
			.find(|s| s.header.kind == StructureKind::BiosInformation)?;
		let info = BiosInformation::from(&stru)?;

		Some(BiosInfo {
			vendor: stru.get_str(info.vendor)?,
			version: stru.get_str(info.version)?,
			release_date: stru.get_str(info.release_date)?,
			major: info.major,
			minor: info.minor
		})
	}

	pub fn system_info(&self) -> Option<SystemInfo> {
		let stru = self.structures.structures()
			.find(|s| s.header.kind == StructureKind::SystemInformation)?;
		let info = SystemInformation::from(&stru)?;

		Some(SystemInfo {
			manufacturer: stru.get_str(info.manufacturer)?,
			product_name: stru.get_str(info.product_name)?,
			version: stru.get_str(info.version)?,
			serial_number: stru.get_str(info.serial_number)?,
			uuid: info.uuid,
			sku_number: stru.get_str(info.sku_number)?,
			family: stru.get_str(info.family)?
		})
	}
}