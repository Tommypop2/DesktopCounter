use strum::{Display, IntoStaticStr};

use crate::tasks::handle_neopixel::RgbMode;

#[derive(Clone, Debug, IntoStaticStr, Display)]
pub enum MenuState {
	/// In the main menu. Display a list of submenus
	Main,
	// Submenus
	RgbMode,
	Test1,
	Test2,
	Test3,
}
impl Menu for MenuState {
	const NUM_OPTIONS: usize = 4;
	fn options<'a>() -> &'a [Self; Self::NUM_OPTIONS] {
		&[
			MenuState::RgbMode,
			MenuState::Test1,
			MenuState::Test2,
			MenuState::Test3,
		]
	}
	fn render(&self, index: usize) -> () {}
}
#[derive(Clone, Debug)]
pub enum State {
	/// Not in a menu. Display the death toll
	DeathToll,
	Menu(MenuState),
}

pub trait Menu {
	const NUM_OPTIONS: usize;
	fn options<'a>() -> &'a [Self; <MenuState as Menu>::NUM_OPTIONS]
	where
		Self: Sized;

	/// Renders the menu with the item at the given index selected
	fn render(&self, index: usize) -> ();
}
