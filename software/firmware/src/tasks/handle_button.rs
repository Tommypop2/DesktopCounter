use core::pin;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{
	gpio::{self, InputConfig, OutputConfig},
	peripherals::{GPIO2, GPIO3},
};
use futures::future::select;
use smart_leds::RGB8;

use crate::tasks::handle_neopixel::{RGB_CONFIG, RgbMode};

pub static BUTTON_STATE: Signal<CriticalSectionRawMutex, ButtonEvent> = Signal::new();
#[derive(Debug, Clone, PartialEq)]
pub enum ButtonEvent {
	Press,
	HoldHalfSecond,
	HoldFullSecond,
}
#[embassy_executor::task]
pub async fn handle_button(led_pin: GPIO2<'static>, button_pin: GPIO3<'static>) {
	let mut led = gpio::Output::new(led_pin, gpio::Level::Low, OutputConfig::default());
	let mut button = gpio::Input::new(button_pin, InputConfig::default().with_pull(gpio::Pull::Up));
	loop {
		button.wait_for_low().await;
		let time_down = Instant::now();
		led.set_high();
		let wait_for_high = pin::pin!(button.wait_for_high());
		let res = select(wait_for_high, Timer::after_millis(500)).await;
		let send = RGB_CONFIG.sender();
		let mut rcv = RGB_CONFIG.receiver().unwrap();
		match res {
			futures::future::Either::Left((_value1, _future2)) => {}
			futures::future::Either::Right((_value2, button_release)) => {
				// In this case, the button is being held, so set colour and wait for release

				let previous_mode: RgbMode;
				{
					let mut config = rcv.get().await;
					previous_mode = config.rgb_mode.clone();
					// "White"
					config.set_mode(RgbMode::Static(RGB8::new(190, 240, 255)));
					send.send(config);
				}
				match select(button_release, Timer::after_millis(500)).await {
					// Button released before next 0.5s
					futures::future::Either::Left(_) => {}
					futures::future::Either::Right((_, button_release)) => {
						{
							send.send({
								let mut cfg = rcv.get().await;
								cfg.set_mode(RgbMode::Static(RGB8::new(0, 0, 255)));
								cfg
							})
						}
						button_release.await;
					}
				}
				send.send({
					let mut cfg = rcv.get().await;
					cfg.set_mode(previous_mode);
					cfg
				})
			}
		}

		let duration_pressed = Instant::now() - time_down;
		led.set_low();
		let button_event = if duration_pressed > Duration::from_ticks(25000) {
			if duration_pressed > Duration::from_millis(1000) {
				ButtonEvent::HoldFullSecond
			} else if duration_pressed > Duration::from_millis(500) {
				ButtonEvent::HoldHalfSecond
			} else {
				ButtonEvent::Press
			}
		} else {
			continue;
		};
		esp_println::dbg!("Button Press: ", &button_event);
		BUTTON_STATE.signal(button_event);
	}
}
