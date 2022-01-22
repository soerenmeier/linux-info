//! Produces a subset from the dmidecode command
//! List all 

use linux_info::bios::Bios;


fn main() {

	let bios = Bios::read().expect("failed to read bios info");
	let bios_info = bios.bios_info().expect("failed to get bios info");
	let system_info = bios.system_info().expect("failed to get system info");

	println!("Bios Information");
	println!("\tVendor: {}", bios_info.vendor);
	println!("\tVersion: {}", bios_info.version);
	println!("\tRelease Date: {}", bios_info.release_date);
	println!("\tBIOS Revision: {}.{}", bios_info.major, bios_info.minor);
	println!();

	println!("System Information");
	println!("\tManufacturer: {}", system_info.manufacturer);
	println!("\tProduct Name: {}", system_info.product_name);
	println!("\tVersion: {}", system_info.version);
	println!("\tSerial Number: {}", system_info.serial_number);
	println!("\tUUID: {}", system_info.uuid);
	println!("\tSKU Number: {}", system_info.sku_number);
	println!("\tFamily: {}", system_info.family);

}