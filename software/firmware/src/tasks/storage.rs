//! Handles storing the config and count to the flash memory of the ESP32-C3

use embassy_futures::yield_now;

#[embassy_executor::task]
pub async fn handle_storage() {
	loop {
		yield_now().await;
	}
}
