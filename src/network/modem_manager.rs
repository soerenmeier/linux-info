//! Connect to the ModemManager

use std::time::Duration;
use std::sync::Arc;

use dbus::{Error, Path};
use dbus::blocking::{Connection, Proxy};
use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
use dbus::arg::{RefArg, PropMap};

use mmdbus::modem::Modem as ModemAccess;
use mmdbus::modem_signal::ModemSignal;
use mmdbus::modem_modem3gpp::ModemModem3gpp;
use mmdbus::sim::Sim as SimTrait;

const DBUS_NAME: &str = "org.freedesktop.ModemManager1";
const DBUS_PATH: &str = "/org/freedesktop/ModemManager1";
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
pub struct ModemManager {
	dbus: Dbus
}

impl ModemManager {
	pub fn connect() -> Result<Self, Error> {
		Dbus::connect()
			.map(|dbus| Self { dbus })
	}

	pub fn modems(&self) -> Result<Vec<Modem>, Error> {
		let objects = self.dbus.proxy(DBUS_PATH).get_managed_objects()?;
		let modems = objects.into_iter()
			.map(|(path, _)| {
				Modem {
					dbus: self.dbus.clone(),
					path
				}
			})
			.collect();

		Ok(modems)
	}
}

pub struct Modem {
	dbus: Dbus,
	path: Path<'static>
}

impl Modem {
	/// The equipment manufacturer, as reported by the modem.
	pub fn manufacturer(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).manufacturer()
	}

	/// The equipment model, as reported by the modem.
	pub fn model(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).model()
	}

	/// The description of the carrier-specific configuration (MCFG) in use by
	/// the modem.
	pub fn carrier_configuration(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).carrier_configuration()
	}

	/// The physical modem device reference (ie, USB, PCI, PCMCIA device),
	/// which may be dependent upon the operating system.
	///
	/// In Linux for example, this points to a sysfs path of the usb_device
	/// object.
	///
	/// This value may also be set by the user using the MM_ID_PHYSDEV_UID udev
	/// tag (e.g. binding the tag to a specific sysfs path).
	pub fn device(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).device()
	}

	/// Overall state of the modem, given as a MMModemState value.
	///
	/// If the device's state cannot be determined, MM_MODEM_STATE_UNKNOWN will
	/// be reported.
	pub fn state(&self) -> Result<ModemState, Error> {
		self.dbus.proxy(&self.path).state()
			.map(Into::into)
	}
	
	/// Signal quality in percent (0 - 100) of the dominant access technology
	/// the device is using to communicate with the network. Always 0 for
	/// POTS devices.  
	/// The additional boolean value indicates if the quality value given was
	/// recently taken. 
	pub fn signal_quality(&self) -> Result<(u32, bool), Error> {
		self.dbus.proxy(&self.path).signal_quality()
	}

	/// A pair of MMModemMode values, where the first one is a bitmask
	/// specifying the access technologies (eg 2G/3G/4G) the device is
	/// currently allowed to use when connecting to a network, and the second
	/// one is the preferred mode of those specified as allowed. 
	pub fn current_modes(&self) -> Result<(ModemMode, ModemMode), Error> {
		self.dbus.proxy(&self.path).current_modes()
			.map(|(a, b)| (a.into(), b.into()))
	}

	/// List of MMModemBand values, specifying the radio frequency and
	/// technology bands the device is currently using when connecting to a
	/// network.
	///
	/// It must be a subset of "SupportedBands".
	pub fn current_bands(&self) -> Result<Vec<ModemBand>, Error> {
		self.dbus.proxy(&self.path).current_bands()
			.map(|v| v.into_iter().map(Into::into).collect())
	}

	pub fn signal_setup(&self, rate: u32) -> Result<(), Error> {
		self.dbus.proxy(&self.path).setup(rate)
	}

	/// Available signal information for the CDMA1x access technology.
	pub fn signal_cdma(&self) -> Result<SignalCdma, Error> {
		let data = self.dbus.proxy(&self.path).cdma()?;
		SignalCdma::from_prop_map(data)
			.ok_or_else(|| Error::new_failed("cdma not found"))
	}

	/// Available signal information for the CDMA EV-DO access technology.
	pub fn signal_evdo(&self) -> Result<SignalEvdo, Error> {
		let data = self.dbus.proxy(&self.path).evdo()?;
		SignalEvdo::from_prop_map(data)
			.ok_or_else(|| Error::new_failed("evdo not found"))
	}

	/// Available signal information for the GSM/GPRS access technology.
	pub fn signal_gsm(&self) -> Result<SignalGsm, Error> {
		let data = self.dbus.proxy(&self.path).gsm()?;
		SignalGsm::from_prop_map(data)
			.ok_or_else(|| Error::new_failed("gsm not found"))
	}

	/// Available signal information for the UMTS (WCDMA) access technology.
	pub fn signal_umts(&self) -> Result<SignalUmts, Error> {
		let data = self.dbus.proxy(&self.path).umts()?;
		SignalUmts::from_prop_map(data)
			.ok_or_else(|| Error::new_failed("umts not found"))
	}

	/// Available signal information for the LTE access technology.
	pub fn signal_lte(&self) -> Result<SignalLte, Error> {
		let data = self.dbus.proxy(&self.path).lte()?;
		SignalLte::from_prop_map(data)
			.ok_or_else(|| Error::new_failed("lte not found"))
	}

	/// Available signal information for the 5G access technology.
	pub fn signal_nr5g(&self) -> Result<SignalNr5g, Error> {
		let data = self.dbus.proxy(&self.path).nr5g()?;
		SignalNr5g::from_prop_map(data)
			.ok_or_else(|| Error::new_failed("nr5g not found"))
	}

	/// List of numbers (e.g. MSISDN in 3GPP) being currently handled by this
	/// modem.
	pub fn own_numbers(&self) -> Result<Vec<String>, Error> {
		self.dbus.proxy(&self.path).own_numbers()
	}

	/// The IMEI of the device.
	/// 
	/// ## Note
	/// This interface will only be available once the modem is ready to be
	/// registered in the cellular network. 3GPP devices will require a valid
	/// unlocked SIM card before any of the features in the interface can be
	/// used.
	pub fn imei(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).imei()
	}

	///  Name of the operator to which the mobile is currently registered.
	///
	/// If the operator name is not known or the mobile is not registered to a
	/// mobile network, this property will be an empty string.
	/// 
	/// ## Note
	/// This interface will only be available once the modem is ready to be
	/// registered in the cellular network. 3GPP devices will require a valid
	/// unlocked SIM card before any of the features in the interface can be
	/// used.
	pub fn operator_name(&self) -> Result<String, Error> {
		ModemModem3gpp::operator_name(&self.dbus.proxy(&self.path))
	}

	/// This SIM object is the one used for network registration and data
	/// connection setup.
	pub fn sim(&self) -> Result<Sim, Error> {
		Ok(Sim {
			path: self.dbus.proxy(&self.path).sim()?,
			dbus: self.dbus.clone()
		})
	}
}

pub struct Sim {
	dbus: Dbus,
	path: Path<'static>
}

impl Sim {
	/// The ICCID of the SIM card.
	///
	/// This may be available before the PIN has been entered depending on the
	/// device itself.
	pub fn identifier(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).sim_identifier()
	}

	/// The IMSI of the SIM card, if any.
	pub fn imsi(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).imsi()
	}

	/// The EID of the SIM card, if any.
	pub fn eid(&self) -> Result<String, Error> {
		self.dbus.proxy(&self.path).eid()
	}

	/// The name of the network operator, as given by the SIM card, if known.
	pub fn operator_name(&self) -> Result<String, Error> {
		SimTrait::operator_name(&self.dbus.proxy(&self.path))
	}
}


#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1")
)]
pub enum ModemState {
	/// The modem is unusable.
	Failed = -1,
	/// State unknown or not reportable.
	Unknown = 0,
	/// The modem is currently being initialized.
	Initializing = 1,
	/// The modem needs to be unlocked.
	Locked = 2,
	/// The modem is not enabled and is powered down.
	Disabled = 3,
	/// The modem is currently transitioning to the MM_MODEM_STATE_DISABLED
	/// state.
	Disabling = 4,
	/// The modem is currently transitioning to the MM_MODEM_STATE_ENABLED
	/// state.
	Enabling = 5,
	/// The modem is enabled and powered on but not registered with a network
	/// provider and not available for data connections.
	Enabled = 6,
	/// The modem is searching for a network provider to register with.
	Searching = 7,
	/// The modem is registered with a network provider, and data connections
	/// and messaging may be available for use.
	Registered = 8,
	/// The modem is disconnecting and deactivating the last active packet data
	/// bearer. This state will not be entered if more than one packet data
	/// bearer is active and one of the active bearers is deactivated.
	Disconnecting = 9,
	/// The modem is activating and connecting the first packet data bearer.
	/// Subsequent bearer activations when another bearer is already active
	/// do not cause this state to be entered.
	Connecting = 10,
	/// One or more packet data bearers is active and connected.
	Connected = 11
}

impl From<i32> for ModemState {
	fn from(num: i32) -> Self {
		if num < -1 || num > 11 {
			Self::Unknown
		} else {
			unsafe {
				*(&num as *const i32 as *const Self)
			}
		}
	}
}

const MODE_NONE: u32 = 0;
/// CSD, GSM, and other circuit-switched technologies.
const MODE_CS: u32 = 1 << 0;
/// GPRS, EDGE.
const MODE_2G: u32 = 1 << 1;
/// UMTS, HSxPA.
const MODE_3G: u32 = 1 << 2;
/// LTE.
const MODE_4G: u32 = 1 << 3;
/// 5GNR
const MODE_5G: u32 = 1 << 4;
/// Any mode can be used (only this value allowed for POTS modems).
const MODE_ANY: u32 = u32::MAX;

// not sure if i like it this way?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModemMode(u32);

impl ModemMode {
	pub fn is_any(&self) -> bool {
		self.0 == MODE_ANY
	}

	pub fn is_none(&self) -> bool {
		self.0 == MODE_NONE
	}

	pub fn has_cs(&self) -> bool {
		self.0 & MODE_CS > 0
	}

	pub fn has_2g(&self) -> bool {
		self.0 & MODE_2G > 0
	}

	pub fn has_3g(&self) -> bool {
		self.0 & MODE_3G > 0
	}

	pub fn has_4g(&self) -> bool {
		self.0 & MODE_4G > 0
	}

	pub fn has_5g(&self) -> bool {
		self.0 & MODE_5G > 0
	}
}

impl From<u32> for ModemMode {
	fn from(num: u32) -> Self {
		Self(num)
	}
}

macro_rules! modem_band {
	($($var:ident = $expr:expr),*) => (
		#[repr(u32)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		#[cfg_attr(
			feature = "serde",
			derive(serde1::Serialize, serde1::Deserialize),
			serde(crate = "serde1")
		)]
		pub enum ModemBand {
			$($var = $expr),*
		}

		impl From<u32> for ModemBand {
			fn from(num: u32) -> Self {
				match num {
					$($expr => Self::$var),*,
					_ => Self::Unknown
				}
			}
		}
	)
}


modem_band! {
	Unknown = 0,
	/* GSM/UMTS bands */
	Egsm = 1,
	Dcs = 2,
	Pcs = 3,
	G850 = 4,
	Utran1 = 5,
	Utran3 = 6,
	Utran4 = 7,
	Utran6 = 8,
	Utran5 = 9,
	Utran8 = 10,
	Utran9 = 11,
	Utran2 = 12,
	Utran7 = 13,
	G450 = 14,
	G480 = 15,
	G750 = 16,
	G380 = 17,
	G410 = 18,
	G710 = 19,
	G810 = 20,
	/* LTE bands */
	Eutran1 = 31,
	Eutran2 = 32,
	Eutran3 = 33,
	Eutran4 = 34,
	Eutran5 = 35,
	Eutran6 = 36,
	Eutran7 = 37,
	Eutran8 = 38,
	Eutran9 = 39,
	Eutran10 = 40,
	Eutran11 = 41,
	Eutran12 = 42,
	Eutran13 = 43,
	Eutran14 = 44,
	Eutran17 = 47,
	Eutran18 = 48,
	Eutran19 = 49,
	Eutran20 = 50,
	Eutran21 = 51,
	Eutran22 = 52,
	Eutran23 = 53,
	Eutran24 = 54,
	Eutran25 = 55,
	Eutran26 = 56,
	Eutran27 = 57,
	Eutran28 = 58,
	Eutran29 = 59,
	Eutran30 = 60,
	Eutran31 = 61,
	Eutran32 = 62,
	Eutran33 = 63,
	Eutran34 = 64,
	Eutran35 = 65,
	Eutran36 = 66,
	Eutran37 = 67,
	Eutran38 = 68,
	Eutran39 = 69,
	Eutran40 = 70,
	Eutran41 = 71,
	Eutran42 = 72,
	Eutran43 = 73,
	Eutran44 = 74,
	Eutran45 = 75,
	Eutran46 = 76,
	Eutran47 = 77,
	Eutran48 = 78,
	Eutran49 = 79,
	Eutran50 = 80,
	Eutran51 = 81,
	Eutran52 = 82,
	Eutran53 = 83,
	Eutran54 = 84,
	Eutran55 = 85,
	Eutran56 = 86,
	Eutran57 = 87,
	Eutran58 = 88,
	Eutran59 = 89,
	Eutran60 = 90,
	Eutran61 = 91,
	Eutran62 = 92,
	Eutran63 = 93,
	Eutran64 = 94,
	Eutran65 = 95,
	Eutran66 = 96,
	Eutran67 = 97,
	Eutran68 = 98,
	Eutran69 = 99,
	Eutran70 = 100,
	Eutran71 = 101,
	/* CDMA Band Classes (see 3GPP2 C.S0057-C) */
	CdmaBc0 = 128,
	CdmaBc1 = 129,
	CdmaBc2 = 130,
	CdmaBc3 = 131,
	CdmaBc4 = 132,
	CdmaBc5 = 134,
	CdmaBc6 = 135,
	CdmaBc7 = 136,
	CdmaBc8 = 137,
	CdmaBc9 = 138,
	CdmaBc10 = 139,
	CdmaBc11 = 140,
	CdmaBc12 = 141,
	CdmaBc13 = 142,
	CdmaBc14 = 143,
	CdmaBc15 = 144,
	CdmaBc16 = 145,
	CdmaBc17 = 146,
	CdmaBc18 = 147,
	CdmaBc19 = 148,
	/* Additional UMTS bands:
	*  15-18 reserved
	*  23-24 reserved
	*  27-31 reserved
	*/
	Utran10 = 210,
	Utran11 = 211,
	Utran12 = 212,
	Utran13 = 213,
	Utran14 = 214,
	Utran19 = 219,
	Utran20 = 220,
	Utran21 = 221,
	Utran22 = 222,
	Utran25 = 225,
	Utran26 = 226,
	Utran32 = 232,
	/* All/Any */
	Any = 256
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1", rename = "camelCase")
)]
pub struct SignalCdma {
	/// The CDMA1x RSSI (Received Signal Strength Indication), in dBm
	pub rssi: f64,
	/// The CDMA1x Ec/Io, in dBm
	pub ecio: f64
}

impl SignalCdma {
	fn from_prop_map(prop: PropMap) -> Option<Self> {
		Some(Self {
			rssi: prop.get("rssi")?
				.as_f64()?,
			ecio: prop.get("ecio")?
				.as_f64()?
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1", rename = "camelCase")
)]
pub struct SignalEvdo {
	/// The CDMA EV-DO RSSI (Received Signal Strength Indication), in dBm
	pub rssi: f64,
	/// The CDMA EV-DO Ec/Io, in dBm
	pub ecio: f64,
	/// CDMA EV-DO SINR level, in dB
	pub sinr: f64,
	/// The CDMA EV-DO Io, in dBm
	pub io: f64
}

impl SignalEvdo {
	fn from_prop_map(prop: PropMap) -> Option<Self> {
		Some(Self {
			rssi: prop.get("rssi")?
				.as_f64()?,
			ecio: prop.get("ecio")?
				.as_f64()?,
			sinr: prop.get("sinr")?
				.as_f64()?,
			io: prop.get("io")?
				.as_f64()?
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1", rename = "camelCase")
)]
pub struct SignalGsm {
	/// The GSM RSSI (Received Signal Strength Indication), in dBm
	pub rssi: f64
}

impl SignalGsm {
	fn from_prop_map(prop: PropMap) -> Option<Self> {
		Some(Self {
			rssi: prop.get("rssi")?
				.as_f64()?
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1", rename = "camelCase")
)]
pub struct SignalUmts {
	/// The UMTS RSSI (Received Signal Strength Indication), in dBm
	pub rssi: f64,
	/// The UMTS RSCP (Received Signal Code Power), in dBm
	pub rscp: f64,
	/// The UMTS Ec/Io, in dB
	pub ecio: f64
}

impl SignalUmts {
	fn from_prop_map(prop: PropMap) -> Option<Self> {
		Some(Self {
			rssi: prop.get("rssi")?
				.as_f64()?,
			rscp: prop.get("rscp")?
				.as_f64()?,
			ecio: prop.get("ecio")?
				.as_f64()?
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1", rename = "camelCase")
)]
pub struct SignalLte {
	/// The LTE RSSI (Received Signal Strength Indication), in dBm
	pub rssi: f64,
	/// The LTE RSRQ (Reference Signal Received Quality), in dB
	pub rsrq: f64,
	/// The LTE RSRP (Reference Signal Received Power), in dBm
	pub rsrp: f64,
	/// The LTE S/R ratio, in dB
	pub snr: f64
}

impl SignalLte {
	fn from_prop_map(prop: PropMap) -> Option<Self> {
		Some(Self {
			rssi: prop.get("rssi")?
				.as_f64()?,
			rsrq: prop.get("rsrq")?
				.as_f64()?,
			rsrp: prop.get("rsrp")?
				.as_f64()?,
			snr: prop.get("snr")?
				.as_f64()?
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde1::Serialize, serde1::Deserialize),
	serde(crate = "serde1", rename = "camelCase")
)]
pub struct SignalNr5g {
	pub rsrq: f64,
	pub rsrp: f64,
	pub snr: f64
}

impl SignalNr5g {
	fn from_prop_map(prop: PropMap) -> Option<Self> {
		Some(Self {
			rsrq: prop.get("rsrq")?
				.as_f64()?,
			rsrp: prop.get("rsrp")?
				.as_f64()?,
			snr: prop.get("snr")?
				.as_f64()?
		})
	}
}