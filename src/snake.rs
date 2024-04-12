use crate::consts::SNAKE_NAMES;
use crate::esc::{bg, fg, reset};
use crate::map::{locate_food, Arena, Effect, Food, Strategy, EFFECT_SECONDS};
use crate::math::{cycle_back, ColoredPoint, Direction, Point, Rng};
use std::time::Duration;
use std::{fmt, time::Instant};

pub struct Snake {
  pub name: &'static str,
  pub color: u8,
  pub body: Vec<Point>,
  pub cannibal: Instant,
  head: usize,
  dir: Direction,
  speed: u8,
  delta: Instant,
  alive: bool,
  strat: Strategy,
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
      cannibal: Instant::now() - Duration::from_secs(EFFECT_SECONDS),
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
    let cannibal = self.is_cannibal();

    for (i, p) in self.body.iter().enumerate() {
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

      if cannibal && i == self.head {
        fg(f, 196)?;
      } else {
        fg(f, self.color)?;
      }

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
      snakes[idx].cannibal = Instant::now() - Duration::from_secs(EFFECT_SECONDS);
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
    if self.len() > 3 {
      self.body.remove(self.tail_idx());
      cycle_back(&self.body, &mut self.head);
      true
    } else {
      false
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
        .filter(|&snake| !std::ptr::addr_eq(self, snake) && self.speed + 4 < snake.speed && snake.len() > 3)
        .map(|snake| snake.tail())
        .min_by_key(|tail| self.tail().quick_distance(tail))
        .copied()
      {
        return target;
      }
    }

    match self.strat {
      Strategy::Player => unreachable!("Player has it's own mind"),
      Strategy::Speed => locate_food(food, self.head(), Effect::Speed),
      Strategy::Score => locate_food(food, self.head(), Effect::Nourish),
      Strategy::Eat => food
        .iter()
        .min_by_key(|food| self.head().quick_distance(food))
        .map(|food| food.position)
        .unwrap(),
      Strategy::Kill => {
        if let Some(target) = snakes
          .iter()
          .filter(|&snake| !std::ptr::addr_eq(self, snake) && self.speed + 10 < snake.speed)
          .max_by_key(|snake| snake.len())
          .map(|snake| *snake.head())
        {
          target
        } else {
          locate_food(food, self.head(), Effect::Speed)
        }
      }
      Strategy::Cannibal => locate_food(food, self.head(), if self.is_cannibal() { Effect::Speed } else { Effect::Cannibal }),
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
