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
	RGB8, SmartLedsWriteAsync as _, brightness,
	hsv::{Hsv, hsv2rgb},
};
use strum::IntoStaticStr;

use crate::maths::sin;
struct FibonacciWrapped {
	num1: u8,
	num2: u8,
}
impl FibonacciWrapped {
	pub fn new() -> Self {
		Self { num1: 0, num2: 1 }
	}
	pub fn next(&mut self) -> u8 {
		let next = self.num1.wrapping_add(self.num2);
		self.num1 = self.num2;
		self.num2 = next;
		next
	}
}
#[derive(Clone, Debug, IntoStaticStr)]
pub enum RgbMode {
	SineCycle(f64),
	Discrete(u64),
	Random(u64),
	Fibonacci(u64),
	Static(RGB8),
}
pub static RGB_MODE: Mutex<CriticalSectionRawMutex, RgbMode> = Mutex::new(RgbMode::SineCycle(0.4));
pub async fn temporarily_set_mode<F: Future>(future: F) -> F::Output {
	let previous_mode: RgbMode;
	{
		let mut mode = RGB_MODE.lock().await;
		previous_mode = mode.clone();
		*mode = RgbMode::Static(RGB8::new(255, 255, 255));
	}
	let v = future.await;
	*RGB_MODE.lock().await = previous_mode;
	v
}
#[embassy_executor::task]
pub async fn handle_neopixel(
	rmt_channel: ChannelCreator<Async, 0>,
	pin: GPIO5<'static>,
	rng: RNG<'static>,
) {
	let mut neopixel = { SmartLedsAdapterAsync::new(rmt_channel, pin, smart_led_buffer!(1)) };
	let level = 10;
	let mut rng = Rng::new(rng);
	let mut fib = FibonacciWrapped::new();
	let mut prev_colour = RGB8::new(0, 0, 0);
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
			RgbMode::Fibonacci(delay) => {
				Timer::after_millis(delay).await;
				let colour = Hsv {
					hue: fib.next(),
					sat: 255,
					val: 255,
				};
				hsv2rgb(colour)
			}
			RgbMode::Static(colour) => colour,
		};
		// Diff the colour (don't write to neopixel if the colour is the same as the previous colour)
		if prev_colour == colour {
			embassy_futures::yield_now().await;
			// Timer::after_millis(10).await;
			continue;
		}
		prev_colour = colour;
		neopixel
			.write(brightness([colour].into_iter(), level))
			.await
			.unwrap();
	}
}
