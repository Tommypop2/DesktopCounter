#![no_std]
#![no_main]

use core::sync::atomic::AtomicU32;

use crate::count::{decrement_count, increase_count, increment_count, read_count};
use crate::menustate::{MenuState, State};
use crate::tasks::handle_button::{BUTTON_STATE, ButtonEvent, handle_button};
use crate::tasks::handle_neopixel::handle_neopixel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::iso_8859_9::FONT_10X20;
use embedded_graphics::{
	mono_font::MonoTextStyleBuilder,
	pixelcolor::BinaryColor,
	prelude::Point,
	text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_hal::{
	i2c::{self, master::Config},
	rmt::Rmt,
	time::Rate,
	timer::timg::TimerGroup,
};

use ssd1306::{
	I2CDisplayInterface, Ssd1306Async, mode::DisplayConfigAsync, prelude::DisplayRotation,
	size::DisplaySize128x64,
};
pub mod count;
pub mod maths;
pub mod menustate;
pub mod tasks;
pub static MENU_STATE: Mutex<CriticalSectionRawMutex, State> = Mutex::new(State::DeathToll);
#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
	esp_println::logger::init_logger_from_env();
	let peripherals = esp_hal::init(esp_hal::Config::default());
	esp_println::println!("Init!");
	// let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();

	let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
	esp_hal_embassy::init(timer_group_0.timer0);

	spawner
		.spawn(handle_button(peripherals.GPIO3, peripherals.GPIO8))
		.unwrap();
	let frequency = Rate::from_mhz(80);
	let rmt = Rmt::new(peripherals.RMT, frequency)
		.expect("Failed to initialize RMT0")
		.into_async();
	spawner
		.spawn(handle_neopixel(
			rmt.channel0,
			peripherals.GPIO5,
			peripherals.RNG,
		))
		.unwrap();
	let i2c = i2c::master::I2c::new(peripherals.I2C0, Config::default())
		.unwrap()
		.with_scl(peripherals.GPIO7)
		.with_sda(peripherals.GPIO6)
		.into_async();
	let interface = I2CDisplayInterface::new(i2c);
	let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
		.into_buffered_graphics_mode();
	display.init().await.unwrap();
	let text_style = MonoTextStyleBuilder::new()
		.font(&FONT_10X20)
		.text_color(BinaryColor::On)
		.build();

	let mut buf = [0u8; 10];
	loop {
		// Clone the value and drop the lock immediately (so it can be modified by another task)
		let value = { MENU_STATE.lock().await.clone() };

		match value {
			State::DeathToll => {
				let value = read_count();
				display.clear_buffer();
				Text::with_baseline("Death Toll", Point::zero(), text_style, Baseline::Top)
					.draw(&mut display)
					.unwrap();
				Text::with_baseline(
					format_no_std::show(&mut buf, format_args!("{}", value)).unwrap(),
					Point::new(0, 20),
					text_style,
					Baseline::Top,
				)
				.draw(&mut display)
				.unwrap();

				display.flush().await.unwrap();
				match BUTTON_STATE.wait().await {
					ButtonEvent::Press => {
						increment_count();
					}
					ButtonEvent::HoldHalfSecond => {
						decrement_count();
					}
					ButtonEvent::HoldFullSecond => {
						*MENU_STATE.lock().await = State::Menu(MenuState::Main);
					}
				}
			}
			State::Menu(menu_state) => match menu_state {
				MenuState::Main => {
					let submenus = &[MenuState::RgbMode];
					display.clear_buffer();
					Text::with_baseline("Menu!!", Point::zero(), text_style, Baseline::Top)
						.draw(&mut display)
						.unwrap();
					display.flush().await.unwrap();
					match BUTTON_STATE.wait().await {
						ButtonEvent::Press => {
							// increment_count();
						}
						ButtonEvent::HoldHalfSecond => {
							// decrement_count();
						}
						ButtonEvent::HoldFullSecond => {
							*MENU_STATE.lock().await = State::DeathToll;
						}
					}
				}
				MenuState::RgbMode => {}
			},
		}
	}
}
