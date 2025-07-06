//! Handles serializing and deserializing how the device is configured
//! to a single u8 value which can easily be saved to the ESP32 flash

use sequential_storage::map::Value;

use crate::{
	const_default::ConstDefault,
	menustate::{RgbBrightness, RgbRate},
	tasks::handle_neopixel::{RGB_CONFIG, RgbMode},
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct RgbConfig {
	pub rgb_mode: RgbMode,
	pub rgb_brightness: RgbBrightness,
	pub rgb_rate_modifier: RgbRate,
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
	pub async fn from_environment() -> Self {
		RGB_CONFIG.lock().await.clone()
	}
	pub async fn apply(self) {
		*RGB_CONFIG.lock().await = self
	}
	pub fn set_mode(&mut self, rgb_mode: RgbMode) {
		self.rgb_mode = rgb_mode;
	}
	pub fn set_brightness(&mut self, rgb_brightness: RgbBrightness) {
		self.rgb_brightness = rgb_brightness;
	}
	pub fn set_rate(&mut self, rgb_rate_modifier: RgbRate) {
		self.rgb_rate_modifier = rgb_rate_modifier;
	}
}
impl ConstDefault for RgbConfig {
	const DEFAULT: Self = Self {
		rgb_mode: RgbMode::DEFAULT,
		rgb_brightness: RgbBrightness::DEFAULT,
		rgb_rate_modifier: RgbRate::DEFAULT,
	};
}
impl<'a> Value<'a> for RgbConfig {
	fn serialize_into(
		&self,
		buffer: &mut [u8],
	) -> Result<usize, sequential_storage::map::SerializationError> {
		buffer[..core::mem::size_of::<Self>()].copy_from_slice(unsafe {
			core::slice::from_raw_parts(
				(self as *const Self) as *const u8,
				core::mem::size_of::<Self>(),
			)
		});
		Ok(core::mem::size_of::<Self>())
	}
	fn deserialize_from(
		buffer: &'a [u8],
	) -> Result<Self, sequential_storage::map::SerializationError>
	where
		Self: Sized,
	{
		let data = unsafe {
			&*buffer[..core::mem::size_of::<Self>()]
				.as_ptr()
				.cast::<Self>()
		}
		.clone();
		Ok(data)
	}
}
