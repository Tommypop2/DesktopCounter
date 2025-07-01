//! Handles serializing and deserializing how the device is configured
//! to a single u8 value which can easily be saved to the ESP32 flash

use crate::{
	menustate::{RgbBrightness, RgbRate},
	tasks::handle_neopixel::RgbMode,
};

pub struct RgbConfig {
	rgb_mode: RgbMode,
	rgb_brightness: RgbBrightness,
	rgb_rate_modifier: RgbRate,
}

impl RgbConfig {
	pub fn new(
		rgb_mode: RgbMode,
		rgb_brightness: RgbBrightness,
		rgb_rate_modifier: RgbRate,
	) -> Self {
		Self {
			rgb_mode,
			rgb_brightness,
			rgb_rate_modifier,
		}
	}
}
