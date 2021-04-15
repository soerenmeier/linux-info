
use std::{fmt, io, mem};
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::convert::TryInto;

use byte_parser::{StrParser, ParseIterator};

const DEF_PRECISION: usize = 2;

/// Represents a size for example `1024 kB`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSize {
	bytes: u128
}

impl DataSize {

	// not implemeting FromStr because this is private.
	pub(crate) fn from_str(s: &str) -> Option<Self> {
		let mut iter = StrParser::new(s);
		let float = parse_f64(&mut iter)?;
		// now we need to parse the unit
		let unit = iter.record()
			.consume_to_str()
			.trim();

		let unit = DataSizeUnit::from_str(unit)?;
		Some(Self {
			bytes: unit.to_byte(float)
		})
	}

	pub(crate) fn from_size_bytes(bytes: impl TryInto<u128>) -> Option<Self> {
		bytes.try_into().ok()
			.map(|bytes| Self {bytes})
	}

	/// Convert the data unit into a specific unit.
	pub fn to(self, unit: &DataSizeUnit) -> f64 {
		DataSizeUnit::convert(self.bytes, unit)
	}

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSizeUnit {
	/// Byte
	B,
	/// Kilobyte
	Kb,// 1_000
	/// Megabyte
	Mb,// 1_000_000
	/// Gigabyte
	Gb,// 1_000_000_000
	/// Terabyte
	Tb // 1_000_000_000_000
}

impl DataSizeUnit {

	const fn val(&self) -> u128 {
		match self {
			Self::B => 1,
			Self::Kb => 1_024,
			Self::Mb => 1_024 * 1_024,
			Self::Gb => 1_024 * 1_024 * 1_024,
			Self::Tb => 1_024 * 1_024 * 1_024 * 1_024
		}
	}

	fn from_str(s: &str) -> Option<Self> {
		Some(match s {
			"" => Self::B,
			s if eqs(s, "b") => Self::B,
			s if eqs(s, "kb") => Self::Kb,
			s if eqs(s, "mb") => Self::Mb,
			s if eqs(s, "gb") => Self::Gb,
			s if eqs(s, "tb") => Self::Tb,
			_ => return None
		})
	}

	fn to_byte(&self, val: f64) -> u128 {
		// TODO probably need fix this overflowing
		(val * self.val() as f64) as u128
	}

	fn adjust_to(byte: u128) -> Self {
		match byte {
			b if b < Self::Kb.val() => Self::B,
			b if b < Self::Mb.val() => Self::Kb,
			b if b < Self::Gb.val() => Self::Mb,
			b if b < Self::Tb.val() => Self::Gb,
			_ => Self::Tb
		}
	}

	fn convert(byte: u128, to: &Self) -> f64 {
		byte as f64 / to.val() as f64
	}

	const fn as_str(&self) -> &'static str {
		match self {
			Self::B => "b",
			Self::Kb => "kb",
			Self::Mb => "mb",
			Self::Gb => "gb",
			Self::Tb => "tb"
		}
	}

	// val needs to be already adjusted to self
	fn fmt_val(&self, val: f64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// calculate the precision we wan't to use
		let fract = val.fract();
		let precision = if fract == 0f64 {
			0
		} else {

			let max = f.precision().unwrap_or(DEF_PRECISION);

			if max <= 1 {
				max
			} else {
				calculate_precision(fract, max)
			}
		};

		write!(f, "{:.*} {}", precision, val, self.as_str())
	}

}

fn calculate_precision(mut fract: f64, max: usize) -> usize {
	let max_number = 10usize.pow(max as u32) as f64;

	// round to the max position
	fract *= max_number;
	fract = fract.round();

	// then go slowly back until we hit a fraction
	for m in (1..=max).rev() {
		fract /= 10f64;
		if fract.fract() != 0f64 {
			return m;
		}
	}

	0
}

#[inline(always)]
fn eqs(a: &str, b: &str) -> bool {
	a.eq_ignore_ascii_case(b)
}

impl fmt::Display for DataSize {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let unit = DataSizeUnit::adjust_to(self.bytes);
		let val = DataSizeUnit::convert(self.bytes, &unit);
		unit.fmt_val(val, f)
	}
}

// parses a part of a slice
// Panics if Iterator contains not valid utf8
fn parse_f64<'s, I>(iter: &mut I) -> Option<f64>
where I: ParseIterator<'s> {

	let mut iter = iter.record();

	// consume first digits
	iter.while_byte_fn(u8::is_ascii_digit)
		.consume_at_least(1)
		.ok()?;

	// dot
	let has_dot = iter
		.next_if(|&b| b == b'.')
		.is_some();

	if has_dot {
		// consume next digits
		iter.consume_while_byte_fn(u8::is_ascii_digit);
	}

	iter.to_str()
		.parse().ok()
}

/// Clears the string the writes the entire file to the string.  
/// Does not allocate in advance like std::fs::read_to_string.
pub fn read_to_string_mut(path: impl AsRef<Path>, s: &mut String) -> io::Result<()> {
	s.clear();
	let mut file = File::open(path)?;
	file.read_to_string(s)
		.map(|_| ())
}


fn cstr(path: impl AsRef<Path>) -> io::Result<CString> {
	CString::new(path.as_ref().as_os_str().as_bytes())
		.map_err(From::from)
}

// see https://man7.org/linux/man-pages/man2/fstatfs.2.html
pub fn statfs(path: impl AsRef<Path>) -> io::Result<libc::statfs> {
	unsafe {
		let mut stat = mem::MaybeUninit::<libc::statfs>::uninit();
		let c = cstr(path)?;
		let r = libc::statfs(c.as_ptr(), stat.as_mut_ptr());
		match r {
			0 => Ok(stat.assume_init()),
			-1 => Err(io::Error::last_os_error()),
			_ => panic!("unexpected return value from statfs {:?}", r)
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_size() {
		let size = DataSize::from_str("24576 kB").unwrap();
		assert_eq!(size.to(&DataSizeUnit::Kb), 24576.0);
	}

	#[test]
	fn size_str() {
		// TODO update the formatter
		let s = DataSize::from_str("1024").unwrap();
		assert_eq!(s.to_string(), "1 kb");
		let s = DataSize::from_str("10 kb").unwrap();
		assert_eq!(s.to_string(), "10 kb");
		let s = DataSize::from_str("42.1 mB").unwrap();
		assert_eq!(s.to_string(), "42.1 mb");
		let s = DataSize::from_str("4.22 Gb").unwrap();
		assert_eq!(s.to_string(), "4.22 gb");
		let s = DataSize::from_str("2000 Tb").unwrap();
		assert_eq!(s.to_string(), "2000 tb");
		// and precision
		assert_eq!(format!("{:.0}", DataSize::from_str("1.2 kb").unwrap()), "1 kb");
	}

	#[test]
	fn test_precision() {
		assert_eq!(calculate_precision(0.00005, 4), 4);
		assert_eq!(calculate_precision(0.00001, 4), 0);
		assert_eq!(calculate_precision(0.0001, 4), 4);
		assert_eq!(calculate_precision(0.001, 4), 3);
		assert_eq!(calculate_precision(0.01, 4), 2);
		assert_eq!(calculate_precision(0.1, 4), 1);
		assert_eq!(calculate_precision(0.0, 4), 0);
	}

	#[test]
	fn run_statfs() {
		statfs("/").unwrap();
	}

}