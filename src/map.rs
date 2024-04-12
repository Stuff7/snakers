use crate::esc::{fg, reset};
use crate::math::{Point, Rng};
use crate::snake::Snake;
use std::fmt::Write;
use std::{fmt, ops::Deref, time::Instant};

pub struct Arena {
  pub position: Point,
  pub size: Point,
}

impl Arena {
  pub fn new(x: u8, y: u8, w: u8, h: u8) -> Self {
    Self {
      position: Point::new(x, y),
      size: Point::new(w, h),
    }
  }
}

impl Arena {
  pub fn render(&mut self, f: &mut String, termsize: &Point, food: &mut [Food]) -> fmt::Result {
    const PADDING: Point = Point::new(16, 2);
    if self.position.x + self.size.x + PADDING.x > termsize.x {
      let diff = (self.position.x + self.size.x + PADDING.x) - termsize.x;
      let sub = diff.saturating_sub(self.position.x.saturating_sub(2));
      self.position.x = self.position.x.saturating_sub(diff);
      if sub != 0 {
        self.shrink_width(sub, food);
      }
    }
    if self.position.y + self.size.y + PADDING.y > termsize.y {
      let diff = (self.position.y + self.size.y + PADDING.y) - termsize.y;
      let sub = diff.saturating_sub(self.position.y.saturating_sub(3));
      self.position.y = self.position.y.saturating_sub(diff);
      if sub != 0 {
        self.shrink_height(sub, food);
      }
    }

    if self.position.x < 2 {
      self.position.x = 2;
    }
    if self.position.y < 3 {
      self.position.y = 3;
    }

    writeln!(f, "\x1b[{};{}H╔{:═<3$}╗", self.position.y, self.position.x, "", self.size.x as usize)?;
    for _ in 0..self.size.y {
      writeln!(f, "\x1b[{}C║\x1b[{}C║", self.position.x.saturating_sub(1), self.size.x)?;
    }
    writeln!(f, "\x1b[{}C╚{:═<2$}╝", self.position.x.saturating_sub(1), "", self.size.x as usize)?;
    Ok(())
  }

  pub fn shrink_width(&mut self, n: u8, food: &mut [Food]) {
    self.size.x = std::cmp::max(8, self.size.x.saturating_sub(n));
    let size = self.size.x - 2;
    for food in food {
      if food.x > size {
        food.position.x -= food.x - size;
      }
    }
  }

  pub fn shrink_height(&mut self, n: u8, food: &mut [Food]) {
    self.size.y = std::cmp::max(8, self.size.y.saturating_sub(n));
    let size = (self.size.y - 1) * 2;
    for food in food {
      if food.y > size {
        food.position.y -= food.y - size;
      }
    }
  }
}

#[derive(Clone, Copy)]
pub enum Strategy {
  Player,
  Speed,
  Score,
  Eat,
  Kill,
  Cannibal,
}

impl Strategy {
  pub fn color(&self) -> u8 {
    match self {
      Strategy::Player => 84,
      Strategy::Speed => 51,
      Strategy::Score => 208,
      Strategy::Eat => 195,
      Strategy::Kill => 210,
      Strategy::Cannibal => 190,
    }
  }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Effect {
  None,
  Speed,
  Nourish,
  Cannibal,
}

impl From<usize> for Effect {
  fn from(value: usize) -> Self {
    match value % 4 {
      0 => Effect::None,
      1 => Effect::Speed,
      2 => Effect::Nourish,
      _ => Effect::Cannibal,
    }
  }
}

pub const EFFECT_SECONDS: u64 = 10;

#[derive(Clone, Copy)]
pub struct Food {
  shape: char,
  pub position: Point,
  color: u8,
  effect: Effect,
}

impl Deref for Food {
  type Target = Point;
  fn deref(&self) -> &Self::Target {
    &self.position
  }
}

impl Food {
  pub fn new(effect: Effect, position: Point) -> Self {
    match effect {
      Effect::None => Self {
        shape: '󰉛',
        position,
        color: 41,
        effect,
      },
      Effect::Speed => Self {
        shape: '',
        position,
        color: 226,
        effect,
      },
      Effect::Nourish => Self {
        shape: '󱩡',
        position,
        color: 213,
        effect,
      },
      Effect::Cannibal => Self {
        shape: '',
        position,
        color: 167,
        effect,
      },
    }
  }

  pub fn random(effect: Effect, rng: &mut Rng, end: &Point) -> Self {
    Self::new(effect, Point::random(rng, end))
  }

  pub fn render(&self, f: &mut String, offset: &Point) -> fmt::Result {
    fg(f, self.color)?;
    self.position.offset(offset).render(self.shape, f)?;
    reset(f)
  }

  pub fn apply_effect(&self, snake: &mut Snake) {
    let mut growth = 1;
    match self.effect {
      Effect::None => (),
      Effect::Speed => snake.add_speed(3),
      Effect::Nourish => growth += 1,
      Effect::Cannibal => snake.cannibal = Instant::now(),
    }
    let head = *snake.head();
    snake.body.extend((0..growth).map(|_| head));
  }
}

pub fn locate_food(food: &[Food], head: &Point, effect: Effect) -> Point {
  food
    .iter()
    .filter(|food| food.effect == effect)
    .map(|food| food.position)
    .min_by_key(|food| head.quick_distance(food))
    .unwrap()
}
