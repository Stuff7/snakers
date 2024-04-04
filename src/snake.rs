use std::fmt::{self, Display, Write};

#[derive(Clone, Copy)]
pub struct Point {
  x: u8,
  y: u8,
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
}

pub struct Snake {
  body: Vec<Point>,
  head: usize,
  dir: Direction,
  color: u8,
  pub speed: u8,
}

impl Snake {
  pub fn new(len: usize, x: u8, y: u8) -> Self {
    Self {
      body: Vec::from_iter((0..len).map(|n| Point {
        x: n as u8 + x + 2,
        y: (y << 1) + 2,
      })),
      head: len - 1,
      dir: Direction::Right,
      color: 84,
      speed: 50,
    }
  }

  pub fn render(&self, f: &mut String, arena: &Arena) -> fmt::Result {
    let mut i = self.head;
    clr(f, self.color)?;

    loop {
      let prev_y = self.body[i].y >> 1;
      let prev_x = self.body[cycle_back(&self.body, &mut i)].x;
      let p = &self.body[i];
      let y = p.y >> 1;

      if (arena.x..arena.x + arena.w).contains(&(p.x - 1)) && (arena.y..arena.y + arena.h).contains(&(y - 1)) {
        let c = if y == prev_y && p.x == prev_x {
          '█'
        } else if p.y % 2 == 0 {
          '▀'
        } else {
          '▄'
        };
        write!(f, "\x1b[{};{}H{}", y, p.x, c)?;
      }

      if i == self.head {
        break;
      }
    }

    reset(f)
  }

  pub fn len(&self) -> usize {
    self.body.len()
  }

  pub fn serpentine(&mut self, arena: &Arena) {
    let (x, y) = match self.dir {
      Direction::Up => (0, -1),
      Direction::Right => (1, 0),
      Direction::Down => (0, 1),
      Direction::Left => (-1, 0),
    };
    self.move_head(x, y, arena);
  }

  pub fn steer(&mut self, dir: Direction) {
    self.dir = if self.dir.inverse() == dir { self.dir } else { dir };
  }

  fn move_head(&mut self, x: i8, y: i8, arena: &Arena) {
    let prev_head = self.body[cycle_back(&self.body, &mut self.head)];
    let head = &mut self.body[self.head];
    head.x = prev_head.x.wrapping_add_signed(x);
    head.y = prev_head.y.wrapping_add_signed(y);

    if head.x > arena.x + arena.w {
      head.x = arena.x + 1;
    } else if head.x < arena.x + 1 {
      head.x = arena.x + arena.w;
    }

    if head.y >> 1 > (arena.y + arena.h) {
      head.y = (arena.y << 1) + 2;
    } else if head.y >> 1 < arena.y + 1 {
      head.y = ((arena.y + arena.h) << 1) + 1;
    }
  }
}

fn cycle_back<T>(v: &[T], i: &mut usize) -> usize {
  let r = *i;
  *i = if r == 0 { v.len() - 1 } else { r - 1 };
  r
}

fn clr(f: &mut String, id: u8) -> fmt::Result {
  write!(f, "\x1b[38;5;{id}m")
}

fn reset(f: &mut String) -> fmt::Result {
  write!(f, "\x1b[0m")
}

pub struct Arena {
  pub w: u8,
  pub h: u8,
  pub x: u8,
  pub y: u8,
}

impl Display for Arena {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "\x1b[{};{}H╔{:═<3$}╗", self.y, self.x, "", self.w as usize)?;
    for _ in 0..self.h {
      writeln!(f, "\x1b[{}C║\x1b[{}C║", self.x - 1, self.w)?;
    }
    writeln!(f, "\x1b[{}C╚{:═<2$}╝", self.x - 1, "", self.w as usize)?;
    Ok(())
  }
}
