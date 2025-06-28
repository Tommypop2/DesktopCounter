use strum::{Display, IntoStaticStr};

use crate::tasks::handle_neopixel::RgbMode;

#[derive(Clone, Debug, IntoStaticStr, Display)]
pub enum MainMenuOptions {
	// Submenus
	RgbMode,
	Test1,
	Test2,
	Test3,
}

#[derive(Clone, Debug)]
pub enum State<'a> {
	/// Not in a menu. Display the death toll
	DeathToll,
	Menu(&'a Menu<'a>),
}

pub trait Renderable {
	/// Renders the menu with the item at the given index selected
	fn render(&self, index: usize) -> ();
}
#[derive(Clone, Debug)]
pub struct Menu<'a> {
	pub name: &'a str,
	pub items: &'a [Menu<'a>],
}
impl<'a> Into<&'a str> for Menu<'a> {
	fn into(self) -> &'a str {
		self.name
	}
}
impl<'a> Menu<'a> {
	pub const fn new(name: &'a str, items: &'a [Menu<'a>]) -> Self {
		Self { name, items }
	}
}

pub static MAIN_MENU: Menu<'static> = Menu::new(
	"main",
	&[
		Menu::new("RgbMode", &[]),
		Menu::new("Test1", &[]),
		Menu::new("Test2", &[]),
		Menu::new("Test3", &[]),
	],
);
