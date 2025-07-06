// Idea for DEFAULT value in trait from https://docs.rs/const-default/latest/const_default/trait.ConstDefault.html

pub trait ConstDefault: Sized {
	const DEFAULT: Self;
}
