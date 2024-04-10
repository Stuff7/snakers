use crate::consts::SNAKE_NAMES;
use crate::esc::{bg, fg, reset};
use crate::math::{Direction, Point, Rng};
use std::{
  fmt::{self, Display},
  ops::{Deref, DerefMut},
  time::Instant,
};

pub struct Snake {
  pub name: &'static str,
  pub color: u8,
  body: Vec<Point>,
  head: usize,
  dir: Direction,
  speed: u8,
  delta: Instant,
}

impl Snake {
  pub fn random(len: usize, rng: &mut Rng, end: &Point) -> Self {
    Self {
      name: SNAKE_NAMES[rng.generate(SNAKE_NAMES.len())],
      color: rng.generate(255) as u8,
      body: vec![Point::random(rng, end); len],
      head: len - 1,
      dir: Direction::random(rng),
      speed: rng.within(40, 69) as u8,
      delta: Instant::now(),
    }
  }

  pub fn head(&self) -> &Point {
    &self.body[self.head]
  }

  pub fn speed(&self) -> u8 {
    u8::MAX - self.speed
  }

  pub fn can_move(&mut self) -> bool {
    if self.delta.elapsed().as_millis() >= self.speed as u128 {
      self.delta = Instant::now();
      return true;
    }
    false
  }

  pub fn add_speed(&mut self, speed: u8) {
    self.speed = self.speed.saturating_sub(speed);
  }

  pub fn render(&self, f: &mut String, arena: &Arena, top: &mut Vec<ColoredPoint>, bottom: &mut Vec<ColoredPoint>) -> fmt::Result {
    for p in &self.body {
      let is_top = p.y % 2 == 0;

      let v = if is_top { &mut *top } else { &mut *bottom };
      if let Some(idx) = v.iter().position(|h| p == &h.point) {
        let h = v.swap_remove(idx);
        // This cursor position in the terminal has it's other half already filled so we set the
        // background to the color of that other half to allow multiple colors along the y axis halves
        bg(f, h.color)?;
      } else {
        let mut h = ColoredPoint {
          point: *p,
          color: self.color,
        };

        if is_top {
          h.y += 1;
        } else {
          h.y -= 1;
        }

        let v = if is_top { &mut *bottom } else { &mut *top };
        v.push(h);
      }

      fg(f, self.color)?;
      p.offset(&arena.position).render(if is_top { '▀' } else { '▄' }, f)?;
      reset(f)?;
    }

    reset(f)
  }

  pub fn len(&self) -> usize {
    self.body.len()
  }

  pub fn serpentine(&mut self, arena: &Arena) {
    let (x, y) = self.dir.coords();
    let prev_head = self.body[cycle_back(&self.body, &mut self.head)];
    let head = &mut self.body[self.head];

    head.x = prev_head.x.wrapping_add_signed(x);
    head.y = prev_head.y.wrapping_add_signed(y);

    if head.x == u8::MAX {
      head.x = arena.size.x - 1;
    } else if head.x > arena.size.x - 1 {
      head.x = 0;
    }

    if head.y == u8::MAX {
      head.y = (arena.size.y << 1) - 1;
    } else if head.y > (arena.size.y << 1) - 1 {
      head.y = 0;
    }
  }

  pub fn eat(&mut self, rng: &mut Rng, food: &mut [Food], arena: &Arena) {
    for food in food {
      if self.body[self.head] == food.position {
        food.apply_effect(self);
        food.position.randomize(rng, &arena.size);
        break;
      }
    }
  }

  pub fn steer(&mut self, dir: Direction) {
    self.dir = if self.dir.inverse() == dir { self.dir } else { dir };
  }

  pub fn seek(&mut self, target: &Point, bounds: &Point) {
    let head = &self.body[self.head];
    for nearest in head.nearest_directions(target, bounds) {
      if nearest == self.dir.inverse() {
        continue;
      }
      let next_head = *head + nearest.coords();
      if !self.is_crash(&next_head) {
        self.dir = nearest;
        break;
      }
    }
  }

  pub fn is_crash(&self, head: &Point) -> bool {
    self.body.iter().any(|p| p == head)
  }
}

pub struct ColoredPoint {
  point: Point,
  color: u8,
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

fn cycle_back<T>(v: &[T], i: &mut usize) -> usize {
  let r = *i;
  *i = if r == 0 { v.len() - 1 } else { r - 1 };
  r
}

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

impl Display for Arena {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "\x1b[{};{}H╔{:═<3$}╗", self.position.y, self.position.x, "", self.size.x as usize)?;
    for _ in 0..self.size.y {
      writeln!(f, "\x1b[{}C║\x1b[{}C║", self.position.x - 1, self.size.x)?;
    }
    writeln!(f, "\x1b[{}C╚{:═<2$}╝", self.position.x - 1, "", self.size.x as usize)?;
    Ok(())
  }
}

#[derive(Clone, Copy)]
pub enum Effect {
  None,
  Speed,
  Nourish,
}

#[derive(Clone, Copy)]
pub struct Food {
  shape: char,
  position: Point,
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
      Effect::Speed => snake.add_speed(1),
      Effect::Nourish => growth += 1,
    }
    let head = snake.body[snake.head];
    snake.body.extend((0..growth).map(|_| head));
  }
}
