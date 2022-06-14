//! Produces a subset from the dmidecode command
//! List all 

// use linux_info::bios::Bios;
use linux_info::network::dbus::Dbus;


fn main() {

	let dbus = Dbus::connect().unwrap();

	for device in dbus.devices().unwrap() {
		println!(
			"type {:?} state {:?} path {:?} interface {:?} driver {:?}",
			device.kind().unwrap(),
			device.state().unwrap(),
			device.path().unwrap(),
			device.interface().unwrap(),
			device.driver().unwrap()
		);

		let ipv4 = device.ipv4_config().unwrap()
			.addresses().unwrap();
		println!("addresses {:?}", ipv4);
	}

}