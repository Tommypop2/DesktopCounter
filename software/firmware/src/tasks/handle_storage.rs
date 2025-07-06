//! Handles storing the config and count to the flash memory of the ESP32-C3

use core::pin::pin;

use embassy_futures::{join::join, yield_now};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Timer;
use esp_println::println;
use esp_storage::FlashStorage;
use futures::future::{Either, select};

use crate::{
	config::RgbConfig,
	count::COUNT,
	storage::{FlashRegion, Storage},
	tasks::handle_neopixel::{RGB_CONFIG, RGB_CONFIG_UPDATED},
};
async fn handle_count_storage(flash: &Mutex<CriticalSectionRawMutex, FlashRegion>) {
	let mut count_storage = Storage::<u32>::new(0);
	let saved_count = count_storage.fetch(&mut *flash.lock().await).await;
	println!("Saved count: {:?}", saved_count);
	if let Some(c) = saved_count {
		COUNT.sender().send(c);
	}
	let mut stored_count = saved_count;
	let mut rcv = COUNT.receiver().unwrap();
	let mut new_count: Option<u32> = None;
	loop {
		match select(pin!(rcv.changed()), Timer::after_secs(2)).await {
			// Count changes before timer completes
			Either::Left((count, _timer)) => new_count = Some(count),
			// Timer completes before count changes, so save
			Either::Right(_r) => {
				if let Some(count) = new_count.take()
					&& Some(count) != stored_count
				{
					println!("Saving count as {count}");
					count_storage
						.write(&count, &mut *flash.lock().await)
						.await
						.unwrap();
					stored_count = Some(count);
					println!("Saved count")
				}
			}
		}
	}
}

async fn handle_config_storage(flash: &Mutex<CriticalSectionRawMutex, FlashRegion>) {
	let mut config_storage = Storage::<RgbConfig>::new(1);
	let stored_config = config_storage.fetch(&mut *flash.lock().await).await;
	if let Some(config) = &stored_config {
		*RGB_CONFIG.lock().await = config.clone();
	}
	let mut stored_config = stored_config;
	let mut rcv = RGB_CONFIG_UPDATED.receiver().unwrap();
	loop {
		match select(pin!(rcv.changed()), Timer::after_secs(2)).await {
			// Count changes before timer completes
			Either::Left((_, _timer)) => {}
			// Timer completes before count changes, so save
			Either::Right(_r) => {
				let config = RGB_CONFIG.lock().await.clone();
				if Some(&config) != stored_config.as_ref() {
					println!("Saving config as {:?}", config);
					config_storage
						.write(&config, &mut *flash.lock().await)
						.await
						.unwrap();
					stored_config = Some(config);
					println!("Saved config")
				}
			}
		}
	}
}
#[embassy_executor::task]
pub async fn handle_storage() {
	// Partition for NVS
	// boot:  0 nvs              WiFi data        01 02 00009000 00006000
	// So region is 0x9000..0xF000
	// 0x9000..0xC000 uses half of the NVS region (so the latter region can easily be used in the future to store something else)
	let flash = Mutex::<CriticalSectionRawMutex, _>::new(FlashRegion::new(
		FlashStorage::new(),
		0x9000..0xC000,
	));
	join(handle_count_storage(&flash), handle_config_storage(&flash)).await;
	unreachable!()
}
