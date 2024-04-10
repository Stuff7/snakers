use crate::math::Point;
use std::fmt::{self, Write};

pub fn mv(f: &mut String, p: &Point) -> fmt::Result {
  write!(f, "\x1b[{};{}H", p.y, p.x)
}

pub fn fg(f: &mut String, id: u8) -> fmt::Result {
  write!(f, "\x1b[38;5;{id}m")
}

pub fn bg(f: &mut String, id: u8) -> fmt::Result {
  write!(f, "\x1b[48;5;{id}m")
}

pub fn reset(f: &mut String) -> fmt::Result {
  write!(f, "\x1b[0m")
}
