//! Produces a subset from the dmidecode command
//! List all 

use std::thread;
use std::time::Duration;

// use linux_info::bios::Bios;
use linux_info::network::network_manager::{NetworkManager, DeviceState};
use linux_info::network::modem_manager::ModemManager;


fn main() {

	let dbus = NetworkManager::connect().unwrap();

	for device in dbus.devices().unwrap() {
		let state = device.state().unwrap();
		let kind = device.kind().unwrap();
		let interface = device.interface().unwrap();

		println!("{:?} {:?} {:?}", interface, kind, state);
		if let Ok(apn) = device.modem_apn() {
			println!("- has apn {:?}", apn);
		}

		if matches!(state, DeviceState::Activated) {
			let ipv4 = device.ipv4_config().unwrap()
				.addresses().unwrap();
			println!("- addresses {:?}", ipv4);
		}
	}

	let dbus = ModemManager::connect().unwrap();

	for modem in dbus.modems().unwrap() {
		println!(
			"modem {:?} {:?} {:?}",
			modem.model().unwrap(),
			modem.manufacturer().unwrap(),
			modem.device().unwrap()
		);

		println!(
			"- carrier configuration: {:?}",
			modem.carrier_configuration().unwrap()
		);

		println!(
			"- state: {:?}, signal: {:?}",
			modem.state().unwrap(),
			modem.signal_quality().unwrap()
		);

		let (allowed_modes, preffered_modes) = modem.current_modes().unwrap();
		println!(
			"- allowed modes: 2g: {} 3g: {} 4g: {} 5g: {}",
			allowed_modes.has_2g(),
			allowed_modes.has_3g(),
			allowed_modes.has_4g(),
			allowed_modes.has_5g()
		);

		println!(
			"- prefered modes: 2g: {} 3g: {} 4g: {} 5g: {}",
			preffered_modes.has_2g(),
			preffered_modes.has_3g(),
			preffered_modes.has_4g(),
			preffered_modes.has_5g()
		);

		println!(
			"- bands: {:?}",
			modem.current_bands().unwrap()
		);

		modem.signal_setup(10).unwrap();
		thread::sleep(Duration::from_secs(1));

		if let Ok(cdma) = modem.signal_cdma() {
			println!("- cdma: {:?}", cdma);
		}

		if let Ok(evdo) = modem.signal_evdo() {
			println!("- evdo: {:?}", evdo);
		}

		if let Ok(gsm) = modem.signal_gsm() {
			println!("- gsm: {:?}", gsm);
		}

		if let Ok(umts) = modem.signal_umts() {
			println!("- umts: {:?}", umts);
		}

		if let Ok(lte) = modem.signal_lte() {
			println!("- lte: {:?}", lte);
		}

		if let Ok(nr5g) = modem.signal_nr5g() {
			println!("- nr5g: {:?}", nr5g);
		}
	}

}