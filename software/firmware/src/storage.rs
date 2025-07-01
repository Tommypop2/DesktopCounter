use core::{marker::PhantomData, ops::Range};

use embassy_embedded_hal::adapter::BlockingAsync;
use esp_storage::FlashStorage;
use sequential_storage::{
	cache::NoCache,
	map::{Value, fetch_item},
};

use crate::config::RgbConfig;

pub struct Storage<'a, T: Value<'a>> {
	flash: BlockingAsync<FlashStorage>,
	flash_range: Range<u32>,
	data_buffer: [u8; 128],
	phantom: PhantomData<&'a T>,
}
impl<'a, T: Value<'a>> Storage<'a, T> {
	pub fn new() -> Self {
		let flash = BlockingAsync::new(FlashStorage::new());
		let flash_range = 0x1000..0x3000;
		let data_buffer = [0; 128];
		Self {
			flash,
			flash_range,
			data_buffer,
			phantom: PhantomData,
		}
	}
	pub async fn fetch(&'a mut self) -> Option<T> {
		fetch_item::<u8, T, _>(
			&mut self.flash,
			self.flash_range.clone(),
			&mut NoCache::new(),
			&mut self.data_buffer,
			&42,
		)
		.await
		.unwrap()
	}
	pub async fn write() {}
}
pub async fn store_config(cfg: RgbConfig) {
	// Below copied from sequential-storage example then `BlockingAsync` was added
	let mut flash = BlockingAsync::new(FlashStorage::new());
	// These are the flash addresses in which the crate will operate.
	// The crate will not read, write or erase outside of this range.
	let flash_range = 0x1000..0x3000;
	// We need to give the crate a buffer to work with.
	// It must be big enough to serialize the biggest value of your storage type in,
	// rounded up to to word alignment of the flash. Some kinds of internal flash may require
	// this buffer to be aligned in RAM as well.
	let mut data_buffer = [0; 128];

	// We can fetch an item from the flash. We're using `u8` as our key type and `u32` as our value type.
	// Nothing is stored in it yet, so it will return None.

	assert_eq!(
		fetch_item::<u8, u32, _>(
			&mut flash,
			flash_range.clone(),
			&mut NoCache::new(),
			&mut data_buffer,
			&42,
		)
		.await
		.unwrap(),
		None
	);
}
