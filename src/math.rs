use std::{
  fmt::{self, Display, Write},
  time::SystemTime,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Point {
  pub x: u8,
  pub y: u8,
}

impl Point {
  pub fn new(x: u8, y: u8) -> Self {
    Self { x, y }
  }

  pub fn render<C: Display>(&self, c: C, f: &mut String) -> fmt::Result {
    write!(f, "\x1b[{};{}H{c}", self.y, self.x)
  }

  pub fn offset(&self, p: &Point) -> Point {
    Self {
      x: self.x + p.x + 1,
      y: ((self.y + 2) >> 1) + p.y,
    }
  }

  pub fn randomize(&mut self, rng: &mut Rng, end: &Point) {
    self.x = rng.generate(end.x as usize) as u8;
    self.y = rng.generate((end.y as usize) << 1) as u8;
  }
}

pub struct Rng(usize);

impl Rng {
  pub fn new() -> Self {
    Self(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as usize)
  }

  pub fn generate(&mut self, max: usize) -> usize {
    const LCG_MULT: usize = 1664525;
    const LCG_INCR: usize = 1013904223;
    self.0 ^= (&max as *const usize) as usize;
    self.0 = self.0.wrapping_mul(LCG_MULT).wrapping_add(LCG_INCR);
    self.0 % max
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
  Up,
  Right,
  Down,
  Left,
}

impl Direction {
  pub fn inverse(&self) -> Self {
    match self {
      Direction::Up => Direction::Down,
      Direction::Right => Direction::Left,
      Direction::Down => Direction::Up,
      Direction::Left => Direction::Right,
    }
  }

  pub fn coords(&self) -> (i8, i8) {
    match self {
      Direction::Up => (0, -1),
      Direction::Right => (1, 0),
      Direction::Down => (0, 1),
      Direction::Left => (-1, 0),
    }
  }
}
