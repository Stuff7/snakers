use std::{
  fmt::{self, Display, Write},
  ops::{Add, Deref, DerefMut},
  time::SystemTime,
};

use crate::esc::mv;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Point {
  pub x: u8,
  pub y: u8,
}

impl Point {
  pub fn new(x: u8, y: u8) -> Self {
    Self { x, y }
  }

  pub fn random(rng: &mut Rng, end: &Point) -> Self {
    let mut p = Self { x: 0, y: 0 };
    p.randomize(rng, end);
    p
  }

  pub fn render<C: Display>(&self, c: C, f: &mut String) -> fmt::Result {
    mv(f, self)?;
    write!(f, "{c}")
  }

  pub fn offset(&self, p: &Point) -> Point {
    Self {
      x: self.x + p.x + 1,
      y: ((self.y + 2) >> 1) + p.y,
    }
  }

  pub fn quick_distance(&self, p: &Point) -> u32 {
    self.x.abs_diff(p.x) as u32 + self.y.abs_diff(p.y) as u32
  }

  pub fn distance(&self, p: &Point) -> u32 {
    let dx = (p.x as i32 - self.x as i32) as f32;
    let dy = (p.y as i32 - self.y as i32) as f32;
    dx.hypot(dy) as u32
  }

  pub fn nearest_directions(&self, target: &Point, bounds: &Point) -> [Direction; 4] {
    let direction_h = if self.x > target.x { Direction::Left } else { Direction::Right };
    let distance_h = target.distance(&self.add(direction_h.coords()));

    let direction_v = if self.y > target.y { Direction::Up } else { Direction::Down };
    let distance_v = target.distance(&self.add(direction_v.coords()));

    if distance_h < distance_v {
      if distance_h > ((bounds.x + 1) as u32) >> 1 {
        [direction_h.inverse(), direction_h, direction_v, direction_v.inverse()]
      } else {
        [direction_h, direction_v, direction_v.inverse(), direction_h.inverse()]
      }
    } else if distance_v > bounds.y as u32 + 2 {
      [direction_v.inverse(), direction_v, direction_h, direction_h.inverse()]
    } else {
      [direction_v, direction_h, direction_h.inverse(), direction_v.inverse()]
    }
  }

  pub fn randomize(&mut self, rng: &mut Rng, end: &Point) {
    self.x = rng.generate(end.x as usize) as u8;
    self.y = rng.generate((end.y as usize) << 1) as u8;
  }
}

impl std::ops::Add<(i8, i8)> for Point {
  type Output = Point;
  fn add(self, rhs: (i8, i8)) -> Self::Output {
    Self {
      x: self.x.wrapping_add_signed(rhs.0),
      y: self.y.wrapping_add_signed(rhs.1),
    }
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

pub struct ColoredPoint {
  pub point: Point,
  pub color: u8,
}

impl Deref for ColoredPoint {
  type Target = Point;
  fn deref(&self) -> &Self::Target {
    &self.point
  }
}

impl DerefMut for ColoredPoint {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.point
  }
}

pub fn cycle_back<T>(v: &[T], i: &mut usize) -> usize {
  let r = *i;
  *i = if r == 0 { v.len() - 1 } else { r - 1 };
  r
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
  Up,
  Right,
  Down,
  Left,
}

impl Direction {
  pub fn random(rng: &mut Rng) -> Self {
    match rng.generate(4) {
      0 => Direction::Up,
      1 => Direction::Right,
      2 => Direction::Down,
      _ => Direction::Left,
    }
  }

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
