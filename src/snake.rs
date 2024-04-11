use crate::consts::SNAKE_NAMES;
use crate::esc::{bg, fg, reset};
use crate::math::{cycle_back, ColoredPoint, Direction, Point, Rng};
use std::time::Duration;
use std::{
  fmt::{self, Display},
  ops::Deref,
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
  alive: bool,
  strat: Strategy,
  cannibal: Instant,
}

impl Snake {
  pub fn random(len: usize, strat: Strategy, rng: &mut Rng, end: &Point) -> Self {
    Self {
      name: SNAKE_NAMES[rng.generate(SNAKE_NAMES.len())],
      color: strat.color(),
      body: vec![Point::random(rng, end); len],
      head: len - 1,
      dir: Direction::random(rng),
      speed: 55,
      delta: Instant::now(),
      alive: true,
      strat,
      cannibal: Instant::now() + Duration::from_secs(EFFECT_SECONDS),
    }
  }

  pub fn head(&self) -> &Point {
    &self.body[self.head]
  }

  pub fn head_mut(&mut self) -> &mut Point {
    &mut self.body[self.head]
  }

  pub fn tail_idx(&self) -> usize {
    if self.head == 0 {
      self.body.len() - 1
    } else {
      self.head - 1
    }
  }

  pub fn tail(&self) -> &Point {
    &self.body[self.tail_idx()]
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

  pub fn serpentine(snakes: &mut [Snake], idx: usize, rng: &mut Rng, arena: &Arena) {
    let (x, y) = snakes[idx].dir.coords();
    let prev_head = snakes[idx].body[cycle_back(&snakes[idx].body, &mut snakes[idx].head)];
    let mut head = snakes[idx].body[snakes[idx].head];

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

    let mut killer = None;
    if snakes[idx].alive && Self::is_crash(snakes, idx, &head, &mut killer) {
      snakes[idx].alive = false;
      snakes[idx].speed = 80;
      if let Some(i) = killer {
        let point = *snakes[i].head();
        let score = snakes[idx].len();
        snakes[i].body.extend((0..score).map(|_| point));
      }
    }

    if snakes[idx].alive {
      *snakes[idx].head_mut() = head;
    } else if !snakes[idx].remove_tail() {
      snakes[idx].alive = true;
      snakes[idx].head_mut().randomize(rng, &arena.size)
    }
  }

  pub fn remove_tail(&mut self) -> bool {
    if self.len() < 3 {
      false
    } else {
      self.body.remove(self.tail_idx());
      cycle_back(&self.body, &mut self.head);
      true
    }
  }

  pub fn eat(snakes: &mut [Snake], idx: usize, rng: &mut Rng, food: &mut [Food], arena: &Arena) {
    for food in food {
      if *snakes[idx].head() == food.position {
        food.apply_effect(&mut snakes[idx]);
        food.position.randomize(rng, &arena.size);
        return;
      }
    }

    if snakes[idx].is_cannibal() {
      for i in 0..snakes.len() {
        if idx == i {
          continue;
        }

        if *snakes[idx].head() == *snakes[i].tail() && snakes[i].remove_tail() {
          snakes[idx].body.push(*snakes[idx].head());
          snakes[idx].cannibal = Instant::now();
          return;
        }
      }
    }
  }

  pub fn steer(&mut self, dir: Direction) {
    self.dir = if self.dir.inverse() == dir { self.dir } else { dir };
  }

  pub fn find_target(&self, snakes: &[Snake], food: &[Food]) -> Point {
    if !matches!(self.strat, Strategy::Player) && self.is_cannibal() {
      if let Some(target) = snakes
        .iter()
        .filter(|&snake| !std::ptr::addr_eq(self, snake) && self.speed + 4 < snake.speed && snake.len() > 2)
        .map(|snake| snake.tail())
        .min_by_key(|tail| self.tail().quick_distance(tail))
        .copied()
      {
        return target;
      }
    }

    match self.strat {
      Strategy::Player => unreachable!("Player has it's own mind"),
      Strategy::Speed => locate_food(food, Effect::Speed),
      Strategy::Score => locate_food(food, Effect::Nourish),
      Strategy::Eat => food
        .iter()
        .min_by_key(|food| self.head().quick_distance(food))
        .map(|food| food.position)
        .unwrap(),
      Strategy::Kill => {
        if let Some(target) = snakes
          .iter()
          .filter(|&snake| !std::ptr::addr_eq(self, snake) && self.speed + 8 < snake.speed)
          .map(|snake| snake.head())
          .min_by_key(|head| self.head().quick_distance(head))
          .copied()
        {
          target
        } else {
          locate_food(food, Effect::Speed)
        }
      }
      Strategy::Cannibal => locate_food(food, if self.is_cannibal() { Effect::Speed } else { Effect::Cannibal }),
    }
  }

  pub fn is_cannibal(&self) -> bool {
    self.cannibal.elapsed().as_secs() < EFFECT_SECONDS
  }

  pub fn seek(snakes: &mut [Snake], idx: usize, target: &Point, bounds: &Point) {
    for nearest in snakes[idx].head().nearest_directions(target, bounds) {
      if nearest == snakes[idx].dir.inverse() {
        continue;
      }
      let next_head = *snakes[idx].head() + nearest.coords();
      if !Self::is_crash(snakes, idx, &next_head, &mut None) {
        snakes[idx].dir = nearest;
        break;
      }
    }
  }

  pub fn is_crash(snakes: &[Snake], idx: usize, head: &Point, killer: &mut Option<usize>) -> bool {
    let cannibal = snakes[idx].is_cannibal();

    let ret = snakes.iter().enumerate().any(|(i, snake)| {
      let crashed = snake
        .body
        .iter()
        .enumerate()
        .any(|(i, p)| !(cannibal && i == snake.tail_idx()) && p == head);
      if crashed && idx != i {
        *killer = Some(i);
      }
      crashed
    });

    ret
  }
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

const EFFECT_SECONDS: u64 = 10;

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
      Effect::Speed => snake.add_speed(2),
      Effect::Nourish => growth += 1,
      Effect::Cannibal => snake.cannibal = Instant::now(),
    }
    let head = snake.body[snake.head];
    snake.body.extend((0..growth).map(|_| head));
  }
}

pub fn locate_food(food: &[Food], effect: Effect) -> Point {
  food.iter().find(|food| food.effect == effect).map(|food| food.position).unwrap()
}
