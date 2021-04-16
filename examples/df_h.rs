//! Produces the same output as df -h.  
//! List all 


use linux_info::storage::{MountPoints, MountPoint};


fn main() {
	let mt = MountPoints::read().expect("could not read /proc/self/mountinfo");
	println!("{:<15} {:>10} {:>10} {:>10} {}", "Filesystem", "Size", "Used", "Avail", "Mounted on");
	for point in mt.points() {
		let _ = print_point(point);
	}
}

// return Some if could print
fn print_point(point: MountPoint) -> Option<()> {
	let stat = point.stats().ok()?;

	if !stat.has_blocks() {
		return None
	}

	println!(
		"{:<15} {:>10} {:>10} {:>10} {}",
		point.mount_source()?,
		format!("{:.1}", stat.total()?),
		format!("{:.1}", stat.available()?),
		format!("{:.1}", stat.used()?),
		point.mount_point()?
	);

	Some(())
}