use core::sync::atomic::AtomicU32;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use esp_println::dbg;

pub static COUNT: Watch<CriticalSectionRawMutex, u32, 2> = Watch::new();

pub fn read_count() -> u32 {
	COUNT.try_get().unwrap_or(0)
}

pub fn write_count(x: u32) {
	let snd = COUNT.sender();
	dbg!("Sending...", x);
	snd.send(x);
}

pub fn increase_count(x: i32) {
	let new = (COUNT.try_get().unwrap_or(0) as i32).wrapping_add(x);
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
