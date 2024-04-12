mod consts;
mod esc;
mod game;
mod map;
mod math;
mod snake;

use game::{Game, GameResult};

fn main() -> GameResult {
  Game::new().fps(60).run()
}
