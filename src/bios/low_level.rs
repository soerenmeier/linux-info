/// only supports SMBIOS 3.0 see https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.4.0.pdf
///
/// only allowed to run on 64bit system with DWORD: u32 & QWORD: u64
///
/// The access method is also only available via
/// the files /sys/firmware/dmi/tables/{smbios_entry_point, DMI}

use std::fs::{self, File};
use std::io::{self, Read};
use std::{iter, str};
use simple_bytes::{Bytes, BytesRead, BytesReadRef};
use memchr::memmem;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
	/// Meaning the file could not be found or we don't have enough permission
	EntryPointNotFound,
	/// This probably means we have a SMBIOS version that is not >= 3.0
	AnchorStringIncorrect,
	/// If something is not correct with the entry point struct.
	/// Note the checksum is ignored.
	EntryPointMalformed,
	/// Meaning the file DMI could not be found or we don't have enough
	/// permissions
	StructuresNotFound,
	/// 
	StructuresMalformed
}

impl From<Error> for io::Error {
	fn from(e: Error) -> Self {
		let kind = match e {
			Error::EntryPointNotFound |
			Error::StructuresNotFound => io::ErrorKind::NotFound,
			_ => io::ErrorKind::Other
		};
		Self::new(kind, format!("{:?}", e))
	}
}

const ANCHOR_STRING: [u8; 5] = [0x5f, 0x53, 0x4d, 0x33, 0x5f];
const ENTRY_POINT_PATH: &str = "/sys/firmware/dmi/tables/smbios_entry_point";
const ENTRY_POINT_MIN_LEN: usize = 5 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 4 + 8;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct EntryPoint {
	/// Checksum of the Entry Point Structure (EPS)
	/// This value, when added to all other bytes in the EPS, results in
	/// the value 00h (using 8-bit addition calculations). Values in the
	/// EPS are summed starting at offset 00h, for Entry Point Length
	/// bytes.
	pub checksum: u8,
	/// Length of the Entry Point Structure, starting with the Anchor String
	/// field, in bytes, currently 18h
	pub len: u8,
	/// Major version of this specification implemented in the table
	/// structures (for example, the value is 0Ah for revision 10.22 and
	/// 02h for revision 2.1)
	pub major: u8,
	/// Minor version of this specification implemented in the table
	/// structures (for example, the value is 16h for revision 10.22 and
	/// 01h for revision 2.1)
	pub minor: u8,
	/// Identifies the docrev of this specification implemented in the table
	/// structures (for example, the value is 00h for revision 10.22.0 and
	/// 01h for revision 2.7.1).
	pub docrev: u8,
	/// EPS revision implemented in this structure and identifies the
	/// formatting of offsets 0Bh and beyond as follows:
	/// 00h Reserved for assignment by this specification
	/// 01h Entry Point is based on SMBIOS 3.0 definition;
	/// 02h-FFh Reserved for assignment by this specification;
	/// offsets 0Ch-17h are defined per revision 01h
	pub revision: u8,
	/// Reserved for assignment by this specification, set to 0
	pub reserved: u8,
	/// Maximum size of SMBIOS Structure Table, pointed to by the
	/// Structure Table Address, in bytes. The actual size is guaranteed
	/// to be less or equal to the maximum size.
	pub table_max: u32,
	/// The 64-bit physical starting address of the read-only SMBIOS
	/// Structure Table, which can start at any 64-bit address. This area
	/// contains all of the SMBIOS structures fully packed together.
	pub table_addr: u64
}

macro_rules! structure_kind {
	($($name:ident = $val:expr),*) => {
		#[derive(Debug, PartialEq, Eq)]
		#[allow(dead_code)]
		pub enum StructureKind {
			$($name),*,
			Unknown
		}

		impl StructureKind {
			fn from_u8(num: u8) -> Self {
				match num {
					$($val => Self::$name),*,
					_ => Self::Unknown
				}
			}
		}
	}
}

structure_kind! {
	BiosInformation = 0,
	SystemInformation = 1,
	SystemEnclosure = 3,
	ProcessorInformation = 4,
	CacheInformation = 7,
	SystemSlots = 9,
	PhysicalMemoryArray = 16,
	MemoryDevice = 17,
	MemoryArrayMappedAddress = 19,
	SystemBootInformation = 32
}

const STRUCTURE_HEADER_LEN: usize = 1 + 1 + 2;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct StructureHeader {
	/// Specifies the type of structure. Types 0 through 127 (7Fh) are reserved for and
	/// defined by this specification. Types 128 through 256 (80h to FFh) are available for
	/// system- and OEM-specific information.
	pub kind: StructureKind,// u8
	/// Specifies the length of the formatted area of the structure, starting at the Type field.
	/// The length of the structure’s string-set is not included.
	pub len: u8,
	/// Specifies the structure’s handle, a unique 16-bit number in the range 0 to 0FFFEh
	/// (for version 2.0) or 0 to 0FEFFh (for version 2.1 and later). The handle can be used
	/// with the Get SMBIOS Structure function to retrieve a specific structure; the handle
	/// numbers are not required to be contiguous. For version 2.1 and later, handle
	/// values in the range 0FF00h to 0FFFFh are reserved for use by this specification.
	///
	/// If the system configuration changes, a previously assigned handle might no longer
	/// exist. However, after a handle has been assigned by the BIOS, the BIOS cannot
	/// re-assign that handle number to another structure.
	pub handle: u16
}

/// Each structure shall be terminated by a double-null (0000h)
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Structure<'a> {
	pub header: StructureHeader,
	pub formatted: &'a [u8],
	/// strings are separated with \0
	pub strings: &'a [u8]
}

const STRUCTURES_PATH: &str = "/sys/firmware/dmi/tables/DMI";

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Structures {
	bytes: Vec<u8>
}

const BIOS_INFO_MIN_LEN: usize = 1 + 1 + 2 + 1 + 1 + 4 + 0 + 1 + 1 + 1 + 1;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct BiosInformation<'a> {
	/// String number of the BIOS Vendor’s Name.
	pub vendor: u8,
	/// String number of the BIOS Version. This
	/// value is a free-form string that may contain
	/// Core and OEM version information.
	pub version: u8,
	/// Segment location of BIOS starting address
	/// (for example, 0E800h).
	/// ## Note
	/// The size of the runtime BIOS image can
	/// be computed by subtracting the Starting
	/// Address Segment from 10000h and
	/// multiplying the result by 16.
	pub starting_addr: u16,
	/// String number of the BIOS release date.
	/// The date string, if supplied, is in either
	/// mm/dd/yy or mm/dd/yyyy format. If the year
	/// portion of the string is two digits, the year is
	/// assumed to be 19yy.
	/// ## Note
	/// The mm/dd/yyyy format is required for
	/// SMBIOS version 2.3 and later.
	pub release_date: u8,
	/// Size (n) where 64K * (n+1) is the size of the
	/// physical device containing the BIOS, in
	/// bytes.
	/// FFh - size is 16MB or greater, see Extended
	/// BIOS ROM Size for actual size
	pub rom_size: u8,
	/// Defines which functions the BIOS supports:
	/// PCI, PCMCIA, Flash, etc. (see 7.1.1).
	pub characteristics: u32,
	/// Optional space reserved for future
	/// supported functions. The number of
	/// Extension Bytes that is present is indicated
	/// by the Length in offset 1 minus 12h. See
	/// 7.1.2 for extensions defined for version 2.1
	/// and later implementations. For version 2.4
	/// and later implementations, two BIOS
	/// Characteristics Extension Bytes are defined
	/// (12-13h) and bytes 14-17h are also defined.
	pub characteristics_extension: &'a [u8],
	/// Identifies the major release of the System
	/// BIOS; for example, the value is 0Ah for
	/// revision 10.22 and 02h for revision 2.1.
	/// This field or the System BIOS Minor
	/// Release field or both are updated each time
	/// a System BIOS update for a given system is
	/// released.
	/// If the system does not support the use of
	/// this field, the value is 0FFh for both this field
	/// and the System BIOS Minor Release field.
	pub major: u8,
	/// Identifies the minor release of the System
	/// BIOS; for example, the value is 16h for
	/// revision 10.22 and 01h for revision 2.1.
	pub minor: u8,
	/// Identifies the major release of the
	/// embedded controller firmware; for example,
	/// the value would be 0Ah for revision 10.22
	/// and 02h for revision 2.1.
	/// This field or the Embedded Controller
	/// Firmware Minor Release field or both are
	/// updated each time an embedded controller
	/// firmware update for a given system is
	/// released.
	/// If the system does not have field
	/// upgradeable embedded controller firmware,
	/// the value is 0FFh.
	pub emc_major: u8,
	/// Identifies the minor release of the
	/// embedded controller firmware; for example,
	/// the value is 16h for revision 10.22 and 01h
	/// for revision 2.1.
	/// If the system does not have field
	/// upgradeable embedded controller firmware,
	/// the value is 0FFh.
	pub emc_minor: u8
}

const SYSTEM_INFO_MIN_LEN: usize = 1 + 1 + 1 + 1 + 16 + 1 + 1 + 1;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SystemInformation {
	/// Number of null-terminated string
	pub manufacturer: u8,
	/// Number of null-terminated string
	pub product_name: u8,
	/// Number of null-terminated string
	pub version: u8,
	/// Number of null-terminated string
	pub serial_number: u8,
	/// Universal unique ID number; see 7.2.1.
	/// is exacly 16 of length
	pub uuid: Uuid,
	/// Identifies the event that caused the system to
	/// power up. See 7.2.2
	pub wake_up_kind: u8,
	/// Number of null-terminated string
	/// This text string identifies a particular computer
	/// configuration for sale. It is sometimes also
	/// called a product ID or purchase order number.
	/// This number is frequently found in existing
	/// fields, but there is no standard format.
	/// Typically for a given system board from a
	/// given OEM, there are tens of unique
	/// processor, memory, hard drive, and optical
	/// drive configurations.
	pub sku_number: u8,
	/// Number of null-terminated string
	/// This text string identifies the family to which a
	/// particular computer belongs. A family refers to
	/// a set of computers that are similar but not
	/// identical from a hardware or software point of
	/// view. Typically, a family is composed of
	/// different computer models, which have
	/// different configurations and pricing points.
	/// Computers in the same family often have
	/// similar branding and cosmetic features.
	pub family: u8
}


impl EntryPoint {
	/// Only the anchor string is checked
	pub fn read() -> Result<Self> {

		let mut buf = [0u8; ENTRY_POINT_MIN_LEN];
		{
			let mut file = File::open(ENTRY_POINT_PATH)
				.map_err(|_| Error::EntryPointNotFound)?;
			file.read_exact(&mut buf)
				.map_err(|_| Error::EntryPointMalformed)?;
			// drop file
		}
		let mut bytes = Bytes::from(buf.as_slice());

		// let's check if we have the correct version
		if bytes.read(ANCHOR_STRING.len()) != ANCHOR_STRING {
			return Err(Error::AnchorStringIncorrect)
		}

		Ok(EntryPoint {
			checksum: bytes.read_le_u8(),
			len: bytes.read_le_u8(),
			major: bytes.read_le_u8(),
			minor: bytes.read_le_u8(),
			docrev: bytes.read_le_u8(),
			revision: bytes.read_le_u8(),
			reserved: bytes.read_le_u8(),
			table_max: bytes.read_le_u32(),
			table_addr: bytes.read_le_u64()
		})
	}
}

impl Structures {
	/// if table_max === 0 the size of DMI is just used
	pub fn read(table_max: u32) -> Result<Self> {
		let buf = fs::read(STRUCTURES_PATH)
			.map_err(|_| Error::StructuresNotFound)?;

		if table_max != 0 && buf.len() > table_max as usize {
			return Err(Error::StructuresMalformed)
		}

		Ok(Self { bytes: buf })
	}

	pub fn structures(&self) -> impl Iterator<Item=Structure> {
		let mut bytes = Bytes::from(self.bytes.as_ref());
		iter::from_fn(move || {
			Structure::read(&mut bytes)
		})
	}
}

impl<'a> Structure<'a> {

	/// Returns null if there is not enough space
	/// or not 2 null bytes where found
	fn read(reader: &mut impl BytesReadRef<'a>) -> Option<Self> {
		if reader.remaining().len() < STRUCTURE_HEADER_LEN {
			return None
		}

		// read header
		let header = StructureHeader {
			kind: StructureKind::from_u8(reader.read_le_u8()),
			len: reader.read_le_u8(),
			handle: reader.read_le_u16()
		};

		// min STRUCTURE_HEADER_LEN since the len contains the header len
		// but we ignore it if len === 0
		let formatted_len = (header.len as usize).max(STRUCTURE_HEADER_LEN)
			- STRUCTURE_HEADER_LEN;

		// +2 since we need \0\0 to end the structure
		if reader.remaining_ref().len() < formatted_len + 2 {
			return None
		}

		let formatted = reader.read_ref(formatted_len);
		let end_pos = memmem::find(reader.remaining(), &[0u8, 0u8])?;
		let strings = reader.read_ref(end_pos);
		let _null_null = reader.read(2);

		Some(Structure { header, formatted, strings })
	}

}

impl<'a> Structure<'a> {
	pub fn get_str(&self, num: u8) -> Option<&'a str> {
		self.strings.split(|b| *b == 0)
			.nth((num.max(1) as usize) - 1)
			.map(str::from_utf8)?
			.ok()
	}
}

impl<'a> BiosInformation<'a> {
	pub fn from(stru: &Structure<'a>) -> Option<Self> {
		debug_assert_eq!(stru.header.kind, StructureKind::BiosInformation);
		debug_assert_eq!(BIOS_INFO_MIN_LEN + STRUCTURE_HEADER_LEN, 0x12);

		if (stru.header.len as usize) < BIOS_INFO_MIN_LEN + STRUCTURE_HEADER_LEN {
			return None
		}

		let char_ext_len = stru.header.len - 0x12;
		let mut bytes = Bytes::from(stru.formatted);

		Some(Self {
			vendor: bytes.read_le_u8(),
			version: bytes.read_le_u8(),
			starting_addr: bytes.read_le_u16(),
			release_date: bytes.read_le_u8(),
			rom_size: bytes.read_le_u8(),
			characteristics: bytes.read_le_u32(),
			characteristics_extension: bytes.read_ref(char_ext_len as usize),
			major: bytes.read_le_u8(),
			minor: bytes.read_le_u8(),
			emc_major: bytes.read_le_u8(),
			emc_minor: bytes.read_le_u8()
		})
	}
}

impl SystemInformation {
	pub fn from(stru: &Structure) -> Option<Self> {
		debug_assert_eq!(stru.header.kind, StructureKind::SystemInformation);

		if (stru.header.len as usize) < SYSTEM_INFO_MIN_LEN
			+ STRUCTURE_HEADER_LEN
		{
			return None
		}

		let mut bytes = Bytes::from(stru.formatted);

		Some(Self {
			manufacturer: bytes.read_le_u8(),
			product_name: bytes.read_le_u8(),
			version: bytes.read_le_u8(),
			serial_number: bytes.read_le_u8(),
			uuid: Uuid::from_fields_le(
				bytes.read_le_u32().to_be(),
				bytes.read_le_u16().to_be(),
				bytes.read_le_u16().to_be(),
				bytes.read(8)
			).unwrap(),
			wake_up_kind: bytes.read_le_u8(),
			sku_number: bytes.read_le_u8(),
			family: bytes.read_le_u8()
		})
	}
}

// Todo add test to make sure that entry_point_min_len >= EntryPoint