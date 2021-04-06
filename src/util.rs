
use std::str::FromStr;
use std::{mem, fmt};
use byte_parser::{StrParser, ParseIterator};

/// Represents a size for example `1024 kB`.
#[derive(Debug, Clone, PartialEq)]
pub enum Size {
	/// byte
	B(f64),
	/// kilobyte (kB)
	Kb(f64),
	/// megabyte (mB)
	Mb(f64),
	/// gigabyte (gB)
	Gb(f64)
}

const BYTE: f64 = 1024f64;

impl Size {

	/// Converts self to bytes.
	pub fn to_b(&mut self) -> f64 {
		let n = match *self {
			Self::B(b) => b,
			Self::Kb(kb) => kb * BYTE,
			Self::Mb(mb) => mb * BYTE * BYTE,
			Self::Gb(gb) => gb * BYTE * BYTE * BYTE
		};
		mem::swap(self, &mut Self::B(n));
		n
	}

	/// Converts self to kilobytes.
	pub fn to_kb(&mut self) -> f64 {
		let n = match *self {
			Self::B(b) => b / BYTE,
			Self::Kb(kb) => kb,
			Self::Mb(mb) => mb * BYTE,
			Self::Gb(gb) => gb * BYTE * BYTE
		};
		mem::swap(self, &mut Self::Kb(n));
		n
	}

	/// Converts self to megabytes.
	pub fn to_mb(&mut self) -> f64 {
		let n = match *self {
			Self::B(b) => b / BYTE / BYTE,
			Self::Kb(kb) => kb / BYTE,
			Self::Mb(mb) => mb,
			Self::Gb(gb) => gb * BYTE
		};
		mem::swap(self, &mut Self::Mb(n));
		n
	}

	/// Converts self to gigabytes.
	pub fn to_gb(&mut self) -> f64 {
		let n = match *self {
			Self::B(b) => b / BYTE / BYTE / BYTE,
			Self::Kb(kb) => kb / BYTE / BYTE,
			Self::Mb(mb) => mb / BYTE,
			Self::Gb(gb) => gb
		};
		mem::swap(self, &mut Self::Gb(n));
		n
	}

}

impl FromStr for Size {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, ()> {
		let mut iter = StrParser::new(s);
		let float = parse_f64(&mut iter)
			.ok_or(())?;
		// now we need to parse the unit
		let unit = iter.record().consume_to_str().trim();
		Ok(match unit {
			"kB" | "kb" => Size::Kb(float),
			"mB" | "mb" => Size::Mb(float),
			"gB" | "gb" => Size::Gb(float),
			u if u.len() == 0 => Size::B(float),
			_ => return Err(())
		})
	}
}

impl fmt::Display for Size {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::B(b) => b.fmt(f),
			Self::Kb(kb) => write!(f, "{} kB", kb),
			Self::Mb(mb) => write!(f, "{} mB", mb),
			Self::Gb(gb) => write!(f, "{} gB", gb)
		}
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


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_size() {
		let size: Size = "24576 kB".parse().unwrap();
		assert_eq!(size, Size::Kb(24576.0));
	}

	#[test]
	fn size_kb() {
		let mut size = Size::B(BYTE);
		size.to_kb();
		assert_eq!(size, Size::Kb(1.0));
	}

	#[test]
	fn size_str() {
		let s = Size::B(BYTE);
		assert_eq!(s.to_string(), "1024");
		let s = Size::Kb(10.0);
		assert_eq!(s.to_string(), "10 kB");
		let s = Size::Mb(42.0);
		assert_eq!(s.to_string(), "42 mB");
		let s = Size::Gb(4.2);
		assert_eq!(s.to_string(), "4.2 gB");
	}

}