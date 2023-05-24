use std::fmt::{self, Display};

pub struct Position {
	pub x: f64,
	pub y: f64,
}

impl Position {
	pub fn new(x: f64, y: f64) -> Position {
		return Position {x, y};
	}
}

impl Display for Position {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(f, "x:{}, y:{}", self.x, self.y);
	}
}

pub struct Moveable {
	pub dx: f64,
	pub dy: f64,
}

impl Moveable {
	pub fn new(dx: f64, dy: f64) -> Moveable {
		return Moveable {dx, dy};
	}
}

impl fmt::Display for Moveable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(f, "dx:{}, dy:{}", self.dx, self.dy);
	}
}

pub struct Collidable {
	width: f64,
	height: f64,
}

pub struct Drawable {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Drawable {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Drawable {
        return Drawable {x, y, w, h};
    }
}
