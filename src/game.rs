use crate::{
  esc::{fg, mv, reset},
  math::{ColoredPoint, Direction, Rng},
  snake::{Arena, Effect, Food, Snake, Strategy},
};
use std::{
  fmt::{self, Display, Write},
  io,
  time::{Duration, Instant},
};

pub struct Game {
  rng: Rng,
  top_halves: Vec<ColoredPoint>,
  bottom_halves: Vec<ColoredPoint>,
  arena: Arena,
  delta: Instant,
  running: bool,
  paused: bool,
  frame_duration_us: u128,
  debug: bool,
  frame: String,
}

const CLEAR: &str = "\x1b[?25l\x1b[2J";
const TIME_US: u128 = 1_000_000;

impl Game {
  pub fn new() -> Self {
    Self {
      rng: Rng::new(),
      top_halves: Vec::with_capacity(1 << 7),
      bottom_halves: Vec::with_capacity(1 << 7),
      arena: Arena::new(2, 2, 32, 15),
      delta: Instant::now(),
      running: false,
      paused: true,
      frame_duration_us: TIME_US / 30,
      debug: false,
      frame: String::from(CLEAR),
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
    let mut snakes: [Snake; 6] = [
      Strategy::Player,
      Strategy::Eat,
      Strategy::Kill,
      Strategy::Speed,
      Strategy::Score,
      Strategy::Cannibal,
    ]
    .map(|strat| {
      let mut snake = Snake::random(8, strat, &mut self.rng, &self.arena.size);
      if matches!(strat, Strategy::Player) {
        snake.name = "You";
      }
      snake
    });
    let mut food = [
      Food::random(Effect::None, &mut self.rng, &self.arena.size),
      Food::random(Effect::Speed, &mut self.rng, &self.arena.size),
      Food::random(Effect::Nourish, &mut self.rng, &self.arena.size),
      Food::random(Effect::Cannibal, &mut self.rng, &self.arena.size),
    ];

    while self.running {
      self.handle_input(&mut snakes[0])?;
      let delta = self.delta.elapsed().as_micros();

      if !self.paused {
        for i in 0..snakes.len() {
          if snakes[i].can_move() {
            if i != 0 {
              let target = snakes[i].find_target(&snakes, &food);
              Snake::seek(&mut snakes, i, &target, &self.arena.size);
            }
            Snake::eat(&mut snakes, i, &mut self.rng, &mut food, &self.arena);
            Snake::serpentine(&mut snakes, i, &mut self.rng, &self.arena);
          }
        }
      }

      if delta >= self.frame_duration_us {
        write!(&mut self.frame, "{}", self.arena)?;

        for snake in &snakes {
          snake.render(&mut self.frame, &self.arena, &mut self.top_halves, &mut self.bottom_halves)?;
        }

        for food in &food {
          food.render(&mut self.frame, &self.arena.position)?;
        }

        self.render_ui(&snakes[0])?;
        self.render_scoreboard(&snakes)?;
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

  fn render_scoreboard(&mut self, snakes: &[Snake]) -> fmt::Result {
    let mut scores: Box<[(u8, &str, usize)]> = snakes.iter().map(|snake| (snake.color, snake.name, snake.len())).collect();
    scores.sort_by_key(|(_, _, score)| usize::MAX - *score);
    let mut position = self.arena.position + (self.arena.size.x as i8 + 2, 1);
    for (color, name, score) in scores.iter() {
      mv(&mut self.frame, &position)?;
      fg(&mut self.frame, *color)?;
      position.y += 1;
      writeln!(&mut self.frame, "{name}: {score}")?;
    }
    reset(&mut self.frame)
  }

  fn handle_input(&mut self, player: &mut Snake) -> GameResult {
    match readln::getch(0) {
      Ok(b) => match b {
        b'w' => player.steer(Direction::Up),
        b'd' => player.steer(Direction::Right),
        b's' => player.steer(Direction::Down),
        b'a' => player.steer(Direction::Left),
        66 => self.arena.position.y = self.arena.position.y.saturating_add(1),
        65 => self.arena.position.y = self.arena.position.y.saturating_sub(1),
        67 => self.arena.position.x = self.arena.position.x.saturating_add(1),
        68 => self.arena.position.x = self.arena.position.x.saturating_sub(1),
        b'k' => self.arena.size.y = self.arena.size.y.saturating_add(1),
        b'j' => self.arena.size.y = self.arena.size.y.saturating_sub(1),
        b'l' => self.arena.size.x = self.arena.size.x.saturating_add(1),
        b'h' => self.arena.size.x = self.arena.size.x.saturating_sub(1),
        b'f' => self.debug = !self.debug,
        b'p' => self.paused = !self.paused,
        b'q' => self.running = false,
        _ => (),
      },
      Err(err) if err.kind() == io::ErrorKind::WouldBlock => (),
      Err(err) => return Err(GameError::Io(err)),
    }

    Ok(())
  }

  fn render_ui(&mut self, player: &Snake) -> fmt::Result {
    mv(&mut self.frame, &(self.arena.position + (0, -2)))?;
    if self.debug {
      let fps = TIME_US / self.delta.elapsed().as_micros();
      write!(
        &mut self.frame,
        "{fps} FPS | T: {:03} | B: {:03}",
        self.top_halves.len(),
        self.bottom_halves.len(),
      )?;
    } else {
      write!(&mut self.frame, "Press F for Debug information")?;
    }

    if self.paused {
      let mut center = self.arena.position + ((self.arena.size.x as i8 / 2) - 22, 0);
      fg(&mut self.frame, 84)?;
      for ln in LOGO.lines() {
        mv(&mut self.frame, &center)?;
        write!(&mut self.frame, "{}", ln)?;
        center.y += 1;
      }
      mv(&mut self.frame, &center)?;
      reset(&mut self.frame)?;
      write!(&mut self.frame, "\x1b[1mControls \x1b[5m(Press P to Play)\x1b[0m")?;
      for c in CONTROLS {
        center.y += 1;
        mv(&mut self.frame, &center)?;
        write!(&mut self.frame, "{c}")?;
      }
    }

    mv(&mut self.frame, &(self.arena.position + (0, -1)))?;
    write!(
      &mut self.frame,
      "SPEED: {}/255 | SCORE: {} | COORDS: {:03}:{:03} | ARENA SIZE: {:03}:{:03} {:?}",
      player.speed(),
      player.len(),
      player.head().x,
      player.head().y,
      self.arena.size.x,
      self.arena.size.y,
      player.cannibal.elapsed().as_secs(),
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

const LOGO: &str = r#"
  ██████  ███▄    █  ▄▄▄       ██ ▄█▀▓█████ 
▒██    ▒  ██ ▀█   █ ▒████▄     ██▄█▒ ▓█   ▀ 
░ ▓██▄   ▓██  ▀█ ██▒▒██  ▀█▄  ▓███▄░ ▒███   
  ▒   ██▒▓██▒  ▐▌██▒░██▄▄▄▄██ ▓██ █▄ ▒▓█  ▄ 
▒██████▒▒▒██░   ▓██░ ▓█   ▓██▒▒██▒ █▄░▒████▒
▒ ▒▓▒ ▒ ░░ ▒░   ▒ ▒  ▒▒   ▓▒█░▒ ▒▒ ▓▒░░ ▒░ ░
░ ░▒  ░ ░░ ░░   ░ ▒░  ▒   ▒▒ ░░ ░▒ ▒░ ░ ░  ░
░  ░  ░     ░   ░ ░   ░   ▒   ░ ░░ ░    ░   
      ░           ░       ░  ░░  ░      ░  ░
"#;

const CONTROLS: [&str; 15] = [
  "\x1b[1mP\x1b[0m -> Play / Pause",
  "\x1b[1mQ\x1b[0m -> Quit",
  "\x1b[1mW\x1b[0m -> Move Up",
  "\x1b[1mD\x1b[0m -> Move Right",
  "\x1b[1mS\x1b[0m -> Move Down",
  "\x1b[1mA\x1b[0m -> Move Left",
  "\x1b[1mK\x1b[0m -> Increase Arena Height",
  "\x1b[1mJ\x1b[0m -> Decrease Arena Height",
  "\x1b[1mL\x1b[0m -> Increase Arena Width",
  "\x1b[1mH\x1b[0m -> Decrease Arena Width",
  "\x1b[1mF\x1b[0m -> Show Debug Info",
  "\x1b[1m\x1b[0m -> Move Arena Down",
  "\x1b[1m\x1b[0m -> Move Arena Up",
  "\x1b[1m\x1b[0m -> Move Arena Right",
  "\x1b[1m\x1b[0m -> Move Arena Left",
];
