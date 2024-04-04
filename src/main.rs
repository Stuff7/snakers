mod game;
mod snake;

use game::{Game, GameResult};

fn main() -> GameResult {
  Game::new(32, 4, 4).fps(60).resize_arena(64, 25).move_arena(4, 4).run()
}
