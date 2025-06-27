#[derive(Clone)]
pub enum MenuState {
	/// In the main menu. Display a list of submenus
	Main,
	// Submenus
	RgbMode,

}

#[derive(Clone)]
pub enum State {
	/// Not in a menu. Display the death toll
	DeathToll,
	Menu(MenuState),
}
