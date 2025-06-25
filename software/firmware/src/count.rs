use core::sync::atomic::AtomicU32;

pub static COUNT: AtomicU32 = AtomicU32::new(0);

pub fn read_count() -> u32 {
	COUNT.load(core::sync::atomic::Ordering::Relaxed)
}

pub fn write_count(x: u32) {
	COUNT.store(x, core::sync::atomic::Ordering::Relaxed)
}

pub fn increase_count(x: i32) {
	let new = (read_count() as i32).wrapping_add(x);
	if new < 0 {
		return;
	}
	write_count(new as u32);
}

pub fn increment_count() {
	increase_count(1);
}

pub fn decrement_count() {
	increase_count(-1);
}
