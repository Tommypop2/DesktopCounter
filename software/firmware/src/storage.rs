use core::{marker::PhantomData, ops::Range};

use embassy_embedded_hal::adapter::BlockingAsync;
use embedded_storage::nor_flash::ReadNorFlash as _;
use esp_storage::FlashStorage;
use sequential_storage::{
	cache::{KeyPointerCache, NoCache},
	map::{Value, fetch_item, store_item},
};

use crate::config::RgbConfig;

/// Storage for a single type, T
pub struct Storage<'a, T: Value<'a>> {
	flash: BlockingAsync<FlashStorage>,
	flash_range: Range<u32>,
	data_buffer: [u8; 128],
	cache: KeyPointerCache<{ page_count() }, u8, 1>,
	search_key: u8,
	phantom: PhantomData<&'a T>,
}
const fn page_count() -> usize {
	const CAPACITY: usize = 4194304;
	CAPACITY / FlashStorage::SECTOR_SIZE as usize
}
impl<'a, T: Value<'a>> Storage<'a, T> {
	/// MUST ensure that `search_key` is unique for this type
	pub fn new(flash: FlashStorage, flash_range: Range<u32>, search_key: u8) -> Self {
		esp_println::println!("Capacity: {} bytes", flash.capacity());
		let cache: KeyPointerCache<{ page_count() }, u8, 1> = KeyPointerCache::new();
		let flash = BlockingAsync::new(flash);
		let data_buffer = [0; 128];
		Self {
			flash,
			flash_range,
			cache,
			search_key,
			data_buffer,
			phantom: PhantomData,
		}
	}
	pub async fn fetch(&'a mut self) -> Option<T> {
		fetch_item::<u8, T, _>(
			&mut self.flash,
			self.flash_range.clone(),
			&mut self.cache,
			&mut self.data_buffer,
			&self.search_key,
		)
		.await
		.unwrap()
	}
	pub async fn write(
		&mut self,
		value: &T,
	) -> Result<(), sequential_storage::Error<esp_storage::FlashStorageError>> {
		store_item::<u8, T, _>(
			&mut self.flash,
			self.flash_range.clone(),
			&mut self.cache,
			&mut self.data_buffer,
			&self.search_key,
			value,
		)
		.await
	}
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
