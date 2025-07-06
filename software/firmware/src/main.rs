#![no_std]
#![no_main]

use crate::count::{COUNT, decrement_count, increment_count, read_count};
use crate::menustate::{MAIN_MENU, MenuResult, MenuType, State, default_index};
use crate::storage::{FlashRegion, Storage};
use crate::tasks::handle_button::{BUTTON_STATE, ButtonEvent, handle_button};
use crate::tasks::handle_neopixel::{
	RGB_BRIGHTNESS, RGB_MODE, RGB_RATE_MULTIPLIER, handle_neopixel,
};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_futures::select::Either;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
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

use esp_storage::FlashStorage;
use ssd1306::{
	I2CDisplayInterface, Ssd1306Async, mode::DisplayConfigAsync, prelude::DisplayRotation,
	size::DisplaySize128x64,
};
pub mod config;
pub mod count;
pub mod maths;
pub mod menustate;
pub mod storage;
pub mod tasks;

pub static MENU_STATE: Mutex<CriticalSectionRawMutex, State> = Mutex::new(State::DeathToll);

#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
	esp_println::logger::init_logger_from_env();
	let peripherals = esp_hal::init(esp_hal::Config::default());
	esp_println::println!("Init!");

	let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
	esp_hal_embassy::init(timer_group_0.timer0);

	spawner
		.spawn(handle_button(peripherals.GPIO2, peripherals.GPIO3))
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

	let mut buf = [0u8; 30];
	let mut menu_index: usize = 0;
	let mut flash = FlashRegion::new(FlashStorage::new(), 0x1000..0x2000);
	let x = Storage::<u32>::new(0);
	loop {
		// Clone the value and drop the lock immediately (so it can be modified by another task)
		let value = { MENU_STATE.lock().await.clone() };

		match value {
			State::DeathToll => {
				let value = COUNT.try_get().unwrap_or(0);
				display.clear_buffer();
				Text::with_baseline("Death Toll", Point::zero(), text_style, Baseline::Top)
					.draw(&mut display)
					.unwrap();
				Text::with_baseline(
					format_no_std::show(&mut buf, format_args!("{value}")).unwrap(),
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
						*MENU_STATE.lock().await = State::Menu(&MAIN_MENU);
					}
				}
			}
			State::Menu(menu) => {
				let items = &menu.items;
				match items {
					Either::First(x) => {
						render_list(x, menu_index, &mut display, &mut buf, text_style).await;
						match BUTTON_STATE.wait().await {
							ButtonEvent::Press => {
								menu_index += 1;
								if menu_index >= x.len() {
									menu_index = 0
								}
							}
							ButtonEvent::HoldHalfSecond => {
								let new_menu = &x[menu_index];
								let new_index = default_index(new_menu).await;
								*MENU_STATE.lock().await = State::Menu(new_menu);
								menu_index = new_index;
							}
							ButtonEvent::HoldFullSecond => {
								*MENU_STATE.lock().await = State::DeathToll;
							}
						}
					}
					Either::Second(x) => {
						render_list(x, menu_index, &mut display, &mut buf, text_style).await;
						match BUTTON_STATE.wait().await {
							ButtonEvent::Press => {
								menu_index += 1;
								if menu_index >= x.len() {
									menu_index = 0
								}
							}
							ButtonEvent::HoldHalfSecond => {
								let result = x[menu_index].clone();
								match result {
									MenuResult::RgbMode(mode) => {
										*RGB_MODE.lock().await = mode;
									}
									MenuResult::RgbBrightness(brightness) => {
										*RGB_BRIGHTNESS.lock().await = brightness;
									}
									MenuResult::RgbRate(rate) => {
										*RGB_RATE_MULTIPLIER.lock().await = rate;
									}
								}
								// menu_index = 0;
								// *MENU_STATE.lock().await = State::Menu(&MAIN_MENU)
							}
							ButtonEvent::HoldFullSecond => {
								menu_index = 0;
								*MENU_STATE.lock().await = State::DeathToll;
							}
						}
					}
				}
			}
		}
	}
}
async fn render_list<'a, T: Clone + Into<&'a str>>(
	items: &[T],
	index: usize,
	display: &mut Ssd1306Async<
		ssd1306::prelude::I2CInterface<i2c::master::I2c<'_, esp_hal::Async>>,
		DisplaySize128x64,
		ssd1306::mode::BufferedGraphicsModeAsync<DisplaySize128x64>,
	>,
	text_buf: &mut [u8],
	text_style: embedded_graphics::mono_font::MonoTextStyle<'_, BinaryColor>,
) {
	if items.is_empty() {
		return;
	}
	let i: i8 = index as i8;
	// Current value should be rendered in the middle with a <
	let previous_value = items[{
		let index = i - 1;
		if index < 0 {
			items.len() as i8 + index
		} else {
			index
		}
	} as usize]
		.clone();
	let current_value = items[i as usize].clone();
	let next_value = items[{
		let index = i + 1;
		if index >= items.len() as i8 {
			index - items.len() as i8
		} else {
			index
		}
	} as usize]
		.clone();
	display.clear_buffer();
	Text::with_baseline(
		previous_value.into(),
		Point::zero(),
		text_style,
		Baseline::Top,
	)
	.draw(display)
	.unwrap();
	Text::with_baseline(
		format_no_std::show(
			text_buf,
			format_args!("{} <", <T as Into<&str>>::into(current_value)),
		)
		.unwrap(),
		Point::new(0, 20),
		text_style,
		Baseline::Top,
	)
	.draw(display)
	.unwrap();
	Text::with_baseline(
		next_value.into(),
		Point::new(0, 40),
		text_style,
		Baseline::Top,
	)
	.draw(display)
	.unwrap();
	display.flush().await.unwrap();
}
