use embassy_time::{Duration, Instant};
use esp_hal::{
	gpio::{self, InputConfig, OutputConfig},
	peripherals::{GPIO2, GPIO8},
};

use crate::count::{decrement_count, increment_count};

#[derive(Debug)]
enum ButtonEvent {
	Press,
	Hold,
}
#[embassy_executor::task]
pub async fn handle_button(led_pin: GPIO2<'static>, button_pin: GPIO8<'static>) {
	let mut led = gpio::Output::new(led_pin, gpio::Level::Low, OutputConfig::default());
	let mut button = gpio::Input::new(button_pin, InputConfig::default().with_pull(gpio::Pull::Up));
	loop {
		// Timer::after(Duration::from_millis(500)).await;
		button.wait_for_low().await;
		let time_down = Instant::now();
		// esp_println::println!("Button pressed!");
		led.set_high();
		button.wait_for_high().await;
		// esp_println::println!("Button released!");
		let duration_pressed = Instant::now() - time_down;
		led.set_low();
		let button_event = if duration_pressed > Duration::from_ticks(25000) {
			// esp_println::println!("Button Registered {}", duration_pressed);
			if duration_pressed > Duration::from_secs(1) {
				ButtonEvent::Hold
			} else {
				ButtonEvent::Press
			}
		} else {
			continue;
		};
		esp_println::dbg!("Button Registered", &button_event);
		match button_event {
			ButtonEvent::Press => increment_count(),
			ButtonEvent::Hold => decrement_count(),
		}
	}
}
