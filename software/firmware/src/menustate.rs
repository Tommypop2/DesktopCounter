use core::mem::MaybeUninit;

use crate::{const_default::ConstDefault, tasks::handle_neopixel::{RgbMode, RGB_BRIGHTNESS, RGB_MODE, RGB_RATE_MULTIPLIER}};
use embassy_futures::select::Either;
use strum::{EnumDiscriminants, IntoDiscriminant, IntoStaticStr, VariantArray};

#[derive(Clone, Debug)]
pub enum State<'a> {
	/// Not in a menu. Display the death toll
	DeathToll,
	Menu(&'a Menu<'a>),
}

pub trait Renderable {
	/// Renders the menu with the item at the given index selected
	fn render(&self, index: usize);
}
type MenuItems<'a> = Either<&'a [Menu<'a>], &'a [MenuResult]>;
#[derive(Clone, Debug)]
pub struct Menu<'a> {
	pub name: &'a str,
	pub items: MenuItems<'a>,
}
impl<'a> From<Menu<'a>> for &'a str {
	fn from(val: Menu<'a>) -> Self {
		val.name
	}
}
impl<'a> Menu<'a> {
	pub const fn new(name: &'a str, items: MenuItems<'a>) -> Self {
		Self { name, items }
	}
}

pub static MAIN_MENU: Menu<'static> = Menu::new(
	"main",
	Either::First(&[
		Menu::new(
			"RGB Mode",
			Either::Second(&[
				MenuResult::RgbMode(RgbMode::SineCycle(0.01)),
				MenuResult::RgbMode(RgbMode::Continuous(1)),
				MenuResult::RgbMode(RgbMode::Random(1)),
				MenuResult::RgbMode(RgbMode::Fibonacci(1)),
			]),
		),
		Menu::new(
			"Brightness",
			Either::Second(&RgbBrightness::map_to_menu_result()),
		),
		Menu::new("RGB Rate", Either::Second(&RgbRate::map_to_menu_result())),
	]),
);
#[derive(Debug, Clone, EnumDiscriminants, PartialEq)]
#[strum_discriminants(name(MenuType))]
pub enum MenuResult {
	RgbMode(RgbMode),
	RgbBrightness(RgbBrightness),
	RgbRate(RgbRate),
}
#[derive(Debug, Clone, Copy, IntoStaticStr, VariantArray, PartialEq)]
pub enum RgbBrightness {
	Low = 10,
	Medium = 100,
	High = 200,
	Max = 255,
}
impl ConstDefault for RgbBrightness {
	const DEFAULT: Self = Self::Low;
}
/// Values roughly model an exponential curve (rounded to the nearest integer)
#[derive(Debug, Clone, Copy, IntoStaticStr, VariantArray, PartialEq)]
pub enum RgbRate {
	VerySlow = 1,
	Slow = 3,
	Moderate = 7,
	Fast = 20,
	VeryFast = 55,
}
impl ConstDefault for RgbRate {
	const DEFAULT: Self = Self::Moderate;
}
/// Nasty macro that allows for a constant mapping of `T` to `MenuResult<T>`
macro_rules! implement_map_to_menu_result {
	($x:ident) => {
		impl $x {
			pub const fn map_to_menu_result() -> [MenuResult; $x::VARIANTS.len()] {
				let mut s = [const { MaybeUninit::<MenuResult>::uninit() }; $x::VARIANTS.len()];
				let mut i = 0;
				while i < $x::VARIANTS.len() {
					s[i].write(MenuResult::$x($x::VARIANTS[i]));
					i += 1;
				}
				// Safe as MaybeUnit<MenuResult> is guaranteed to have the same size and alignment as MenuResult
				unsafe { s.as_ptr().cast::<[MenuResult; $x::VARIANTS.len()]>().read() }
			}
		}
	};
}
implement_map_to_menu_result!(RgbBrightness);
implement_map_to_menu_result!(RgbRate);
impl From<MenuResult> for &'static str {
	fn from(value: MenuResult) -> Self {
		match value {
			MenuResult::RgbMode(x) => x.into(),
			MenuResult::RgbBrightness(x) => x.into(),
			MenuResult::RgbRate(x) => x.into(),
		}
	}
}

pub async fn default_index<'a>(m: &Menu<'a>) -> usize {
	if let Either::Second(x) = &m.items {
		let tp = MenuType::from(&x[0]);
		match tp {
			MenuType::RgbMode => {
				let current_mode = { RGB_MODE.lock().await.clone() };
				x.iter()
					.position(|y| *y == MenuResult::RgbMode(current_mode.clone()))
					.unwrap_or(0)
			}
			MenuType::RgbBrightness => {
				let current_mode = { RGB_BRIGHTNESS.lock().await.clone() };
				x.iter()
					.position(|y| *y == MenuResult::RgbBrightness(current_mode.clone()))
					.unwrap_or(0)
			}
			MenuType::RgbRate => {
				let current_mode = { RGB_RATE_MULTIPLIER.lock().await.clone() };
				x.iter()
					.position(|y| *y == MenuResult::RgbRate(current_mode.clone()))
					.unwrap_or(0)
			}
		}
	} else {
		0
	}
}
