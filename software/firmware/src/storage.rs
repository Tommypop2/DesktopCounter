use core::{marker::PhantomData, ops::Range};

use embassy_embedded_hal::adapter::BlockingAsync;
use esp_storage::FlashStorage;
use sequential_storage::{
	cache::{KeyPointerCache, NoCache},
	map::{Value, fetch_item, store_item},
};

use crate::config::RgbConfig;

/// Storage for a single type, T
pub struct Storage<T: for<'a> Value<'a>> {
	data_buffer: [u8; 128],
	search_key: u8,
	phantom: PhantomData<T>,
}
const fn page_count() -> usize {
	const CAPACITY: usize = 4194304;
	CAPACITY / FlashStorage::SECTOR_SIZE as usize
}
/// Region of flash where the data will be stored. Includes a cache for this flash range
pub struct FlashRegion {
	flash: BlockingAsync<FlashStorage>,
	cache: KeyPointerCache<{ page_count() }, u8, 2>,
	flash_range: Range<u32>,
}
impl FlashRegion {
	pub fn new(flash: FlashStorage, flash_range: Range<u32>) -> Self {
		let flash = BlockingAsync::new(flash);
		let cache = KeyPointerCache::new();
		Self {
			flash,
			flash_range,
			cache,
		}
	}
}
impl<T: for<'a> Value<'a>> Storage<T> {
	/// MUST ensure that `search_key` is unique for this type
	pub fn new(search_key: u8) -> Self {
		let data_buffer = [0; 128];
		Self {
			search_key,
			data_buffer,
			phantom: PhantomData,
		}
	}
	pub async fn fetch(&mut self, flash: &mut FlashRegion) -> Option<T> {
		fetch_item::<u8, T, _>(
			&mut flash.flash,
			flash.flash_range.clone(),
			&mut flash.cache,
			&mut self.data_buffer,
			&self.search_key,
		)
		.await
		.unwrap()
	}
	pub async fn write(
		&mut self,
		value: &T,
		flash: &mut FlashRegion,
	) -> Result<(), sequential_storage::Error<esp_storage::FlashStorageError>> {
		store_item::<u8, T, _>(
			&mut flash.flash,
			flash.flash_range.clone(),
			&mut flash.cache,
			&mut self.data_buffer,
			&self.search_key,
			value,
		)
		.await
	}
}
