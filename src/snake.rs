use crate::math::{Direction, Point, Rng};
use std::{
  fmt::{self, Display, Write},
  ops::{Deref, DerefMut},
};

pub struct Snake {
  pub body: Vec<Point>,
  pub head: usize,
  dir: Direction,
  color: u8,
  pub speed: u8,
}

impl Snake {
  pub fn new(len: usize, x: u8, y: u8) -> Self {
    Self {
      body: vec![Point::new(x, y); len],
      head: len - 1,
      dir: Direction::Right,
      color: 84,
      speed: 50,
    }
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

  pub fn eat(&mut self, rng: &mut Rng, food: &mut Point, arena: &Arena) {
    let head = &self.body[self.head];
    if head.x == food.x && head.y == food.y {
      self.body.push(self.body[self.head]);
      food.randomize(rng, &arena.size);
    }
  }

  pub fn steer(&mut self, dir: Direction) {
    self.dir = if self.dir.inverse() == dir { self.dir } else { dir };
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

fn fg(f: &mut String, id: u8) -> fmt::Result {
  write!(f, "\x1b[38;5;{id}m")
}

fn bg(f: &mut String, id: u8) -> fmt::Result {
  write!(f, "\x1b[48;5;{id}m")
}

fn reset(f: &mut String) -> fmt::Result {
  write!(f, "\x1b[0m")
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
