use strum::{Display, IntoStaticStr};

#[derive(Clone, Debug, IntoStaticStr, Display)]
pub enum MenuState {
	/// In the main menu. Display a list of submenus
	Main,
	// Submenus
	RgbMode,
	Test1,
	Test2,
	Test3
}

#[derive(Clone, Debug)]
pub enum State {
	/// Not in a menu. Display the death toll
	DeathToll,
	Menu(MenuState),
}
