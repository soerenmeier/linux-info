//!
//! The data is retrieved from `/proc/cpuinfo`
//!
//! ```
//! use linux_info::cpu::CpuInfo;
//! let info = CpuInfo::load_sync().unwrap();
//! let model_name = info.first_value("model name").unwrap();
//! // or every model name
//! let model_names = info.unique_values("model name");
//! ```
//!
//! To list all availabe key's [linuxwiki.org](https://linuxwiki.org/proc/cpuinfo). Or you can use the api
//! ```
//! use linux_info::cpu::CpuInfo;
//! let info = CpuInfo::load_sync().expect("no cpu info");
//!	let first = info.first().expect("no cpu found");
//! let keys = first.keys();
//! ```


use std::path::Path;
use std::{fs, io};

/// Load cpu info into this struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuInfo {
	raw: String
}

impl CpuInfo {

	fn path() -> &'static Path {
		Path::new("/proc/cpuinfo")
	}

	#[cfg(test)]
	fn from_string(raw: String) -> Self {
		Self {raw}
	}

	/// Load cpu infos synchronously.
	pub fn load_sync() -> io::Result<Self> {
		Ok(Self {
			raw: fs::read_to_string(Self::path())?
		})
	}

	/// Load cpu infos asynchronously.
	#[cfg(feature = "async")]
	pub async fn load_async() -> io::Result<Self> {
		Ok(Self {
			raw: tokio::fs::read_to_string(Self::path()).await?
		})
	}

	/// Main method to get cpu infos. Returns every entry.
	pub fn all_infos<'a>(&'a self) -> impl Iterator<Item=CpuInfoEntry<'a>> {
		self.raw.split("\n\n")
			.map(CpuInfoEntry::from_str)
	}

	/// Returns the first entry.
	pub fn first<'a>(&'a self) -> Option<CpuInfoEntry<'a>> {
		self.all_infos().next()
	}

	/// Returns the amount of cores.
	pub fn cores(&self) -> usize {
		self.all_infos().count()
	}

	/// Returns the value of the first.
	pub fn first_value<'a>(&'a self, key: &str) -> Option<&'a str> {
		self.first()
			.and_then(|i| i.value(key))
	}

	/// Returns the unique values to a specific key.
	pub fn unique_values<'a>(&'a self, key: &str) -> Vec<&'a str> {
		let mut list = vec![];
		self.all_infos()
			.filter_map(|info| info.value(key))
			.for_each(|v| {
				if !list.contains(&v) {
					list.push(v);
				}
			});
		list
	}

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuInfoEntry<'a> {
	raw: &'a str
}

impl<'a> CpuInfoEntry<'a> {

	fn from_str(raw: &'a str) -> Self {
		Self {raw}
	}

	/// returns every key and valu ein the cpu info
	pub fn values(&self) -> impl Iterator<Item=Option<(&'a str, &'a str)>> {
		self.raw.split('\n')
			.map(|line| {
				// TODO: after 1.52 update tot split_once
				let mut iter = line.splitn(2, ':');
				let (key, value) = (iter.next()?, iter.next()?);
				Some((key.trim(), value.trim()))
			})
	}

	/// get a value to it's corresponding key
	pub fn value(&self, key: &str) -> Option<&'a str> {
		self.values()
			.filter_map(|kv| kv)
			.find_map(|(k, v)| k.eq_ignore_ascii_case(key).then(|| v))
	}

	/// list all available keys
	pub fn keys(&self) -> impl Iterator<Item=&'a str> {
		self.values()
			.filter_map(|kv| kv)
			.map(|(k, _)| k)
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	fn cpu_info() -> CpuInfo {
		CpuInfo::from_string("\
processor	: 16
vendor_id	: AuthenticAMD
cpu family	: 23
model		: 113
model name	: AMD Ryzen 9 3900XT 12-Core Processor
stepping	: 0
microcode	: 0x8701021
cpu MHz		: 2196.035
cache size	: 512 KB
physical id	: 0
siblings	: 24
core id		: 6
cpu cores	: 12
apicid		: 13
initial apicid	: 13
fpu		: yes
fpu_exception	: yes
cpuid level	: 16
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl nonstop_tsc cpuid extd_apicid aperfmperf pni pclmulqdq monitor ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand lahf_lm cmp_legacy svm extapic cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw ibs skinit wdt tce topoext perfctr_core perfctr_nb bpext perfctr_llc mwaitx cpb cat_l3 cdp_l3 hw_pstate sme ssbd mba sev ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 cqm rdt_a rdseed adx smap clflushopt clwb sha_ni xsaveopt xsavec xgetbv1 xsaves cqm_llc cqm_occup_llc cqm_mbm_total cqm_mbm_local clzero irperf xsaveerptr rdpru wbnoinvd arat npt lbrv svm_lock nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold avic v_vmsave_vmload vgif umip rdpid overflow_recov succor smca
bugs		: sysret_ss_attrs spectre_v1 spectre_v2 spec_store_bypass
bogomips	: 7586.59
TLB size	: 3072 4K pages
clflush size	: 64
cache_alignment	: 64
address sizes	: 43 bits physical, 48 bits virtual
power management: ts ttp tm hwpstate cpb eff_freq_ro [13] [14]

processor	: 17
vendor_id	: AuthenticAMD
cpu family	: 23
model		: 113
model name	: AMD Ryzen 9 3900XT 12-Core Processor
stepping	: 0
microcode	: 0x8701021
cpu MHz		: 2196.035
cache size	: 512 KB
physical id	: 0
siblings	: 24
core id		: 6
cpu cores	: 12
apicid		: 13
initial apicid	: 13
fpu		: yes
fpu_exception	: yes
cpuid level	: 16
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl nonstop_tsc cpuid extd_apicid aperfmperf pni pclmulqdq monitor ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand lahf_lm cmp_legacy svm extapic cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw ibs skinit wdt tce topoext perfctr_core perfctr_nb bpext perfctr_llc mwaitx cpb cat_l3 cdp_l3 hw_pstate sme ssbd mba sev ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 cqm rdt_a rdseed adx smap clflushopt clwb sha_ni xsaveopt xsavec xgetbv1 xsaves cqm_llc cqm_occup_llc cqm_mbm_total cqm_mbm_local clzero irperf xsaveerptr rdpru wbnoinvd arat npt lbrv svm_lock nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold avic v_vmsave_vmload vgif umip rdpid overflow_recov succor smca
bugs		: sysret_ss_attrs spectre_v1 spectre_v2 spec_store_bypass
bogomips	: 7586.59
TLB size	: 3072 4K pages
clflush size	: 64
cache_alignment	: 64
address sizes	: 43 bits physical, 48 bits virtual
power management: ts ttp tm hwpstate cpb eff_freq_ro [13] [14]\
		".into())
	}

	#[test]
	fn info_to_vec() {
		let cpu_info = cpu_info();
		let v: Vec<_> = cpu_info.all_infos().collect();
		assert_eq!(v.len(), 2);
	}

	#[test]
	fn info_values() {
		let info = cpu_info();
		let mut values = info.all_infos();
		let first = values.next().unwrap();
		println!("first {:?}", first.values().collect::<Vec<_>>());
		let model_name = first.value("model name").unwrap();
		assert_eq!(model_name, "AMD Ryzen 9 3900XT 12-Core Processor");
	}

	#[test]
	fn count_cores() {
		let cpu_info = cpu_info();
		assert_eq!(cpu_info.cores(), 2);
	}

	#[test]
	fn unique_values() {
		let cpu_info = cpu_info();
		let un = cpu_info.unique_values("model name");
		assert_eq!(un.len(), 1);
	}

}