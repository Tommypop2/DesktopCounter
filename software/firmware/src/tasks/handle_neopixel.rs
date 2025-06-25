use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Instant, Timer};
use esp_hal::{
	Async,
	peripherals::{GPIO5, RNG},
	rmt::ChannelCreator,
	rng::Rng,
};
use esp_hal_smartled::{SmartLedsAdapterAsync, smart_led_buffer};
use smart_leds::{
	SmartLedsWriteAsync as _, brightness,
	hsv::{Hsv, hsv2rgb},
};

use crate::maths::sin;

pub enum RgbMode {
	SineCycle(f64),
	Discrete(u64),
	Random(u64),
}
pub static RGB_MODE: Mutex<CriticalSectionRawMutex, RgbMode> = Mutex::new(RgbMode::SineCycle(0.4));
#[embassy_executor::task]
pub async fn handle_neopixel(
	rmt_channel: ChannelCreator<Async, 0>,
	pin: GPIO5<'static>,
	rng: RNG<'static>,
) {
	let mut neopixel = { SmartLedsAdapterAsync::new(rmt_channel, pin, smart_led_buffer!(1)) };
	let level = 10;
	let mut rng = Rng::new(rng);
	loop {
		let colour = match *RGB_MODE.lock().await {
			RgbMode::SineCycle(angular_freq) => {
				let time = Instant::now().as_micros() as f64 / 1E6;
				let colour = Hsv {
					hue: (sin(angular_freq * time) * 255.0) as u8,
					sat: 255,
					val: 255,
				};
				hsv2rgb(colour)
			}
			RgbMode::Discrete(rate) => {
				let time = Instant::now().as_secs();
				let colour = Hsv {
					hue: ((time * rate) % 255) as u8,
					sat: 255,
					val: 255,
				};
				hsv2rgb(colour)
			}
			RgbMode::Random(delay) => {
				Timer::after_millis(delay).await;
				let colour = Hsv {
					hue: (rng.random() / 257) as u8,
					sat: 255,
					val: 255,
				};
				hsv2rgb(colour)
			}
		};

		neopixel
			.write(brightness([colour].into_iter(), level))
			.await
			.unwrap();
	}
}
