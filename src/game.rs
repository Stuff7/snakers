use crate::snake::{Arena, Direction, Snake};
use std::{
  fmt::{self, Display, Write},
  io,
  time::{Duration, Instant},
};

pub struct Game {
  snake: Snake,
  arena: Arena,
  delta: Instant,
  running: bool,
  frame_duration_us: u128,
  visible_fps: bool,
  frame: String,
  pressed: u8,
}

const CLEAR: &str = "\x1b[?25l\x1b[2J";
const TIME_US: u128 = 1_000_000;

impl Game {
  pub fn new(snake_len: usize, x: u8, y: u8) -> Self {
    Self {
      snake: Snake::new(snake_len, x, y),
      arena: Arena { w: 32, h: 15, x: 2, y: 2 },
      delta: Instant::now(),
      running: false,
      frame_duration_us: TIME_US / 30,
      visible_fps: false,
      frame: String::from(CLEAR),
      pressed: 0,
    }
  }

  pub fn resize_arena(&mut self, w: u8, h: u8) -> &mut Self {
    self.arena.w = w;
    self.arena.h = h;
    self
  }

  pub fn move_arena(&mut self, x: u8, y: u8) -> &mut Self {
    self.arena.x = x;
    self.arena.y = y;
    self
  }

  pub fn fps(&mut self, fps: usize) -> &mut Self {
    self.frame_duration_us = TIME_US / fps as u128;
    self
  }

  pub fn run(&mut self) -> GameResult {
    self.running = true;
    let mut snake_delta = Instant::now();

    while self.running {
      self.handle_input()?;
      let delta = self.delta.elapsed().as_micros();

      if snake_delta.elapsed().as_millis() >= self.snake.speed as u128 {
        self.snake.serpentine(&self.arena);
        snake_delta = Instant::now();
      }

      if delta >= self.frame_duration_us {
        self.render_stats()?;
        write!(&mut self.frame, "{}", self.arena)?;
        self.snake.render(&mut self.frame, &self.arena)?;
        println!("{}", self.frame);
        self.frame.truncate(10);
        self.delta = Instant::now() + Duration::from_micros((delta - self.frame_duration_us) as u64);
      }
    }

    println!("\x1b[?25h");
    Ok(())
  }

  fn handle_input(&mut self) -> GameResult {
    match readln::getch(0) {
      Ok(b) => {
        self.pressed = b;
        match b {
          b'w' => self.snake.steer(Direction::Up),
          b'd' => self.snake.steer(Direction::Right),
          b's' => self.snake.steer(Direction::Down),
          b'a' => self.snake.steer(Direction::Left),
          b'+' => self.snake.speed = self.snake.speed.wrapping_sub(1),
          b'-' => self.snake.speed = self.snake.speed.wrapping_add(1),
          66 => self.arena.y = self.arena.y.saturating_add(1),
          65 => self.arena.y = self.arena.y.saturating_sub(1),
          67 => self.arena.x = self.arena.x.saturating_add(1),
          68 => self.arena.x = self.arena.x.saturating_sub(1),
          b'k' => self.arena.h = self.arena.h.saturating_add(1),
          b'j' => self.arena.h = self.arena.h.saturating_sub(1),
          b'l' => self.arena.w = self.arena.w.saturating_add(1),
          b'h' => self.arena.w = self.arena.w.saturating_sub(1),
          b'f' => self.visible_fps = !self.visible_fps,
          b'q' => self.running = false,
          _ => (),
        }
      }
      Err(err) if err.kind() == io::ErrorKind::WouldBlock => (),
      Err(err) => return Err(GameError::Io(err)),
    }

    Ok(())
  }

  fn render_stats(&mut self) -> fmt::Result {
    write!(&mut self.frame, "\x1b[{};{}H", self.arena.y - 1, self.arena.x)?;
    if self.visible_fps {
      let fps = TIME_US / self.delta.elapsed().as_micros();
      write!(&mut self.frame, "{fps} FPS | ")?;
    }
    write!(
      &mut self.frame,
      "SPEED: {}/255 | SIZE: {} | KEY: {}",
      255 - self.snake.speed,
      self.snake.len(),
      self.pressed
    )
  }
}

pub type GameResult<T = ()> = Result<T, GameError>;

#[derive(Debug)]
pub enum GameError {
  Io(io::Error),
  Fmt(fmt::Error),
}

impl std::error::Error for GameError {}

macro_rules! from_err {
  ($to: ident, $from: ty) => {
    impl From<$from> for GameError {
      fn from(value: $from) -> Self {
        Self::$to(value)
      }
    }
  };
}

from_err!(Io, io::Error);
from_err!(Fmt, fmt::Error);

impl Display for GameError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Io(err) => write!(f, "{err}"),
      Self::Fmt(err) => write!(f, "{err}"),
    }
  }
}
