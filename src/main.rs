mod consts;
mod esc;
mod game;
mod math;
mod snake;

use game::{Game, GameResult};

fn main() -> GameResult {
  Game::new().fps(60).resize_arena(64, 25).move_arena(4, 4).run()
}
