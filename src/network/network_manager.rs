//! Connect to the NetworkManager

use std::time::Duration;
use std::sync::Arc;
use std::net::Ipv4Addr;

use dbus::{Error, Path};
use dbus::blocking::{Connection, Proxy};
use dbus::arg::RefArg;

use nmdbus::NetworkManager as DbusNetworkManager;
use nmdbus::device::Device as DeviceTrait;
use nmdbus::device_modem::DeviceModem;
use nmdbus::ip4config::IP4Config;

const DBUS_NAME: &str = "org.freedesktop.NetworkManager";
const DBUS_PATH: &str = "/org/freedesktop/NetworkManager";
const TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
struct Dbus {
	conn: Arc<Connection>
}

impl Dbus {
	fn connect() -> Result<Self, Error> {
		Connection::new_system()
			.map(Arc::new)
			.map(|conn| Self { conn })
	}

	fn proxy<'a, 'b>(
		&'b self,
		path: impl Into<Path<'a>>
	) -> Proxy<'a, &'b Connection> {
		self.conn.with_proxy(DBUS_NAME, path, TIMEOUT)
	}
}

#[derive(Clone)]
pub struct NetworkManager {
	dbus: Dbus
}

impl NetworkManager {
	pub fn connect() -> Result<Self, Error> {
		Dbus::connect()
			.map(|dbus| Self { dbus })
	}

	pub fn devices(&self) -> Result<Vec<Device>, Error> {
		let paths = self.dbus.proxy(DBUS_PATH).get_devices()?;
		let devices = paths.into_iter()
			.map(|path| {
				Device {
					dbus: self.dbus.clone(),
					path
				}
			})
			.collect();

		Ok(devices)
	}
}

pub struct Device {
	dbus: Dbus,
	path: Path<'static>
}

impl Device {
	/// The path of the device as exposed by the udev property ID_PATH.  
	/// Note that non-UTF-8 characters are backslash escaped.
	/// Use g_strcompress() to obtain the true (non-UTF-8) string. 
	pub fn path(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).path()
	}

	/// The name of the device's control (and often data) interface. Note that
	/// non UTF-8 characters are backslash escaped, so the resulting name may
	/// be longer then 15 characters. Use g_strcompress() to revert the
	/// escaping.
	pub fn interface(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).interface()
	}

	/// The driver handling the device. Non-UTF-8 sequences are backslash
	/// escaped. Use g_strcompress() to revert. 
	pub fn driver(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).driver()
	}

	/// The current state of the device. 
	pub fn state(&self) -> Result<DeviceState, Error> {
		DeviceTrait::state(&self.dbus.proxy(&self.path))
			.map(Into::into)
	}

	/// The general type of the network device; ie Ethernet, Wi-Fi, etc.
	pub fn kind(&self) -> Result<DeviceKind, Error> {
		self.dbus.proxy(&self.path).device_type()
			.map(Into::into)
	}

	/// Ipv4 Configuration of the device. Only valid when the device is in
	/// DeviceState::Activated
	pub fn ipv4_config(&self) -> Result<Ipv4Config, Error> {
		self.dbus.proxy(&self.path).ip4_config()
			.map(|path| Ipv4Config {
				dbus: self.dbus.clone(),
				path
			})
	}

	/// The access point name the modem is connected to. Blank if disconnected.
	pub fn modem_apn(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).apn()
	}
}

pub struct Ipv4Config {
	dbus: Dbus,
	path: Path<'static>
}

impl Ipv4Config {
	pub fn addresses(&self) -> Result<Vec<Ipv4Addr>, Error> {
		let data = self.dbus.proxy(&self.path).address_data()?;
		let addrs = data.into_iter()
			.filter_map(|mut d| d.remove("address"))
			.filter_map(|addr| {
				addr.as_str()?
					.parse().ok()
			})
			.collect();

		Ok(addrs)
	}
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1")
)]
pub enum DeviceKind {
	/// unknown device
	Unknown = 0,
	/// generic support for unrecognized device types
	Generic = 14,
	/// a wired ethernet device
	Ethernet = 1,
	/// an 802.11 Wi-Fi device
	Wifi = 2,
	/// not used
	Unused1 = 3,
	/// not used
	Unused2 = 4,
	/// a Bluetooth device supporting PAN or DUN access protocols
	Bt = 5,
	/// an OLPC XO mesh networking device
	OlpcMesh = 6,
	/// an 802.16e Mobile WiMAX broadband device
	Wimax = 7,
	/// a modem supporting analog telephone, CDMA/EVDO, GSM/UMTS,
	/// or LTE network access protocols
	Modem = 8,
	/// an IP-over-InfiniBand device
	Infiniband = 9,
	/// a bond master interface
	Bond = 10,
	/// an 802.1Q VLAN interface
	Vlan = 11,
	/// ADSL modem
	Adsl = 12,
	/// a bridge master interface
	Bridge = 13,
	/// a team master interface
	Team = 15,
	/// a TUN or TAP interface
	Tun = 16,
	/// a IP tunnel interface
	IpTunnel = 17,
	/// a MACVLAN interface
	Macvlan = 18,
	/// a VXLAN interface
	Vxlan = 19,
	/// a VETH interface
	Veth = 20,
	/// a MACsec interface
	Macsec = 21,
	/// a dummy interface
	Dummy = 22,
	/// a PPP interface
	Ppp = 23,
	/// a Open vSwitch interface
	OvsInterface = 24,
	/// a Open vSwitch port
	OvsPort = 25,
	/// a Open vSwitch bridge
	OvsBridge = 26,
	/// a IEEE 802.15.4 (WPAN) MAC Layer Device
	Wpan = 27,
	/// 6LoWPAN interface
	SixLowPan = 28,
	/// a WireGuard interface
	Wireguard = 29,
	/// an 802.11 Wi-Fi P2P device. Since: 1.16.
	WifiP2p = 30,
	/// A VRF (Virtual Routing and Forwarding) interface. Since: 1.24.
	Vrf = 31
}

impl From<u32> for DeviceKind {
	fn from(num: u32) -> Self {
		if num > 31 {
			Self::Unknown
		} else {
			unsafe {
				*(&num as *const u32 as *const Self)
			}
		}
	}
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1")
)]
pub enum DeviceState {
	/// the device's state is unknown
	Unknown = 0,
	/// the device is recognized, but not managed by NetworkManager
	Unmanaged = 10,
	/// the device is managed by NetworkManager, but is not available for use.
	/// Reasons may include the wireless switched off, missing firmware, no
	/// ethernet carrier, missing supplicant or modem manager, etc.
	Unavailable = 20,
	/// the device can be activated, but is currently idle and not connected
	/// to a network.
	Disconnected = 30,
	/// the device is preparing the connection to the network. This may include
	/// operations like changing the MAC address, setting physical link
	/// properties, and anything else required to connect to the requested
	/// network.
	Prepare = 40,
	/// the device is connecting to the requested network. This may include
	/// operations like associating with the Wi-Fi AP, dialing the modem,
	/// connecting to the remote Bluetooth device, etc.
	Config = 50,
	/// the device requires more information to continue connecting to the
	/// requested network. This includes secrets like WiFi passphrases, login
	/// passwords, PIN codes, etc.
	NeedAuth = 60,
	/// the device is requesting IPv4 and/or IPv6 addresses and routing
	/// information from the network.
	IpConfig = 70,
	/// the device is checking whether further action is required for the
	/// requested network connection. This may include checking whether only
	/// local network access is available, whether a captive portal is
	/// blocking access to the Internet, etc.
	IpCheck = 80,
	/// the device is waiting for a secondary connection (like a VPN) which
	/// must activated before the device can be activated
	Secondaries = 90,
	/// the device has a network connection, either local or global.
	Activated = 100,
	/// a disconnection from the current network connection was requested, and
	/// the device is cleaning up resources used for that connection. The
	/// network connection may still be valid.
	Deactivating = 110,
	/// the device failed to connect to the requested network and is cleaning
	/// up the connection request
	Failed = 120
}

impl From<u32> for DeviceState {
	fn from(num: u32) -> Self {
		if num > 120 || num % 10 != 0 {
			Self::Unknown
		} else {
			unsafe {
				*(&num as *const u32 as *const Self)
			}
		}
	}
}