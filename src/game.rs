use crate::{
  math::{Direction, Point, Rng},
  snake::{Arena, ColoredPoint, Snake},
};
use std::{
  fmt::{self, Display, Write},
  io,
  time::{Duration, Instant},
};

pub struct Game {
  snake: Snake,
  rng: Rng,
  top_halves: Vec<ColoredPoint>,
  bottom_halves: Vec<ColoredPoint>,
  food: Point,
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
      rng: Rng::new(),
      top_halves: Vec::with_capacity(1 << 7),
      bottom_halves: Vec::with_capacity(1 << 7),
      food: Point::new(17, 0),
      arena: Arena::new(2, 2, 32, 15),
      delta: Instant::now(),
      running: false,
      frame_duration_us: TIME_US / 30,
      visible_fps: false,
      frame: String::from(CLEAR),
      pressed: 0,
    }
  }

  pub fn resize_arena(&mut self, w: u8, h: u8) -> &mut Self {
    self.arena.size.x = w;
    self.arena.size.y = h;
    self
  }

  pub fn move_arena(&mut self, x: u8, y: u8) -> &mut Self {
    self.arena.position.x = x;
    self.arena.position.y = y;
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
        self.snake.eat(&mut self.rng, &mut self.food, &self.arena);
        snake_delta = Instant::now();
      }

      if delta >= self.frame_duration_us {
        write!(&mut self.frame, "{}", self.arena)?;
        self
          .snake
          .render(&mut self.frame, &self.arena, &mut self.top_halves, &mut self.bottom_halves)?;
        self.food.offset(&self.arena.position).render("\x1b[38;5;210m󰉛\x1b[0m", &mut self.frame)?;
        self.render_stats()?;
        println!("{}", self.frame);
        self.top_halves.clear();
        self.bottom_halves.clear();
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
          66 => self.arena.position.y = self.arena.position.y.saturating_add(1),
          65 => self.arena.position.y = self.arena.position.y.saturating_sub(1),
          67 => self.arena.position.x = self.arena.position.x.saturating_add(1),
          68 => self.arena.position.x = self.arena.position.x.saturating_sub(1),
          b'k' => self.arena.size.y = self.arena.size.y.saturating_add(1),
          b'j' => self.arena.size.y = self.arena.size.y.saturating_sub(1),
          b'l' => self.arena.size.x = self.arena.size.x.saturating_add(1),
          b'h' => self.arena.size.x = self.arena.size.x.saturating_sub(1),
          b'f' => self.visible_fps = !self.visible_fps,
          b'q' => self.running = false,
          b'r' => self.food.randomize(&mut self.rng, &self.arena.size),
          _ => (),
        }
      }
      Err(err) if err.kind() == io::ErrorKind::WouldBlock => (),
      Err(err) => return Err(GameError::Io(err)),
    }

    Ok(())
  }

  fn render_stats(&mut self) -> fmt::Result {
    write!(&mut self.frame, "\x1b[{};{}H", self.arena.position.y - 2, self.arena.position.x)?;
    if self.visible_fps {
      let fps = TIME_US / self.delta.elapsed().as_micros();
      write!(&mut self.frame, "{fps} FPS | ")?;
    }
    write!(
      &mut self.frame,
      "SPEED: {}/255 | SIZE: {} | KEY: {}\x1b[{};{}H=HEAD: {:03}:{:03} | FOOD: {:03}:{:03} | ARENA: {:03}:{:03} | T: {:03} | B: {:03}=",
      255 - self.snake.speed,
      self.snake.len(),
      self.pressed,
      self.arena.position.y - 1,
      self.arena.position.x,
      self.snake.body[self.snake.head].x,
      self.snake.body[self.snake.head].y,
      self.food.x,
      self.food.y,
      self.arena.position.x,
      self.arena.position.y,
      self.top_halves.len(),
      self.bottom_halves.len()
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
