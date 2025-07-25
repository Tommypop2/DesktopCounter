use embassy_sync::mutex::Mutex;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Instant;
use esp_hal::{
	Async,
	peripherals::{GPIO5, RNG},
	rmt::ChannelCreator,
	rng::Rng,
};
use esp_hal_smartled::{SmartLedsAdapterAsync, smart_led_buffer};
use smart_leds::{
	RGB8, SmartLedsWriteAsync as _, brightness, gamma,
	hsv::{Hsv, hsv2rgb},
};
use strum::IntoStaticStr;

use crate::{
	config::RgbConfig,
	const_default::ConstDefault,
	maths::{FibonacciWrapped, sin},
};

#[derive(Clone, Debug, IntoStaticStr, PartialEq)]
pub enum RgbMode {
	SineCycle(f32),
	Continuous(u32),
	Random(u32),
	Fibonacci(u32),
	Static(RGB8),
}
impl ConstDefault for RgbMode {
	const DEFAULT: Self = Self::SineCycle(0.01);
}

pub static RGB_CONFIG: Mutex<CriticalSectionRawMutex, RgbConfig> = Mutex::new(RgbConfig::DEFAULT);
pub static RGB_CONFIG_UPDATED: Watch<CriticalSectionRawMutex, u8, 4> = Watch::new();
#[embassy_executor::task]
pub async fn handle_neopixel(
	rmt_channel: ChannelCreator<Async, 0>,
	pin: GPIO5<'static>,
	rng: RNG<'static>,
) {
	let mut neopixel = { SmartLedsAdapterAsync::new(rmt_channel, pin, smart_led_buffer!(1)) };
	let mut rng = Rng::new(rng);
	let mut fib = FibonacciWrapped::new();
	let mut prev_colour = RGB8::new(0, 0, 0);
	loop {
		let config = RGB_CONFIG.lock().await.clone();
		let rate_multiplier = config.rgb_rate_modifier as u8;
		let colour = match config.rgb_mode {
			RgbMode::SineCycle(rate) => {
				let time = Instant::now().as_micros() as f64 / 1E6;
				let colour = Hsv {
					hue: (sin(time * (rate as f64 * rate_multiplier as f64)) * 255.0) as u8,
					sat: 255,
					val: 255,
				};
				hsv2rgb(colour)
			}
			RgbMode::Continuous(rate) => {
				let time = Instant::now().as_micros() as f64 / 1E6;
				let colour = Hsv {
					hue: ((time * rate as f64 * rate_multiplier as f64) as u64 % 255) as u8,
					sat: 255,
					val: 255,
				};
				hsv2rgb(colour)
			}
			RgbMode::Random(rate) => {
				let time = Instant::now().as_millis() as u32;
				if time % (5000 / (rate * rate_multiplier as u32)) == 0 {
					let colour = Hsv {
						hue: (rng.random() / 257) as u8,
						sat: 255,
						val: 255,
					};
					hsv2rgb(colour)
				} else {
					embassy_futures::yield_now().await;
					continue;
				}
			}
			RgbMode::Fibonacci(rate) => {
				let time = Instant::now().as_millis() as u32;
				if time % (5000 / (rate * rate_multiplier as u32)) == 0 {
					let colour = Hsv {
						hue: fib.next(),
						sat: 255,
						val: 255,
					};
					hsv2rgb(colour)
				} else {
					embassy_futures::yield_now().await;
					continue;
				}
			}
			RgbMode::Static(colour) => colour,
		};
		// Diff the colour (don't write to neopixel if the colour is the same as the previous colour)
		if prev_colour == colour {
			embassy_futures::yield_now().await;
			continue;
		}
		prev_colour = colour;
		let level = config.rgb_brightness as u8;
		neopixel
			.write(brightness(gamma([colour].into_iter()), level))
			.await
			.unwrap();
	}
}
