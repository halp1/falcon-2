pub mod game;
pub mod keyfinder;
pub mod search;
pub mod trainer;

use game::{
  Game, GameConfig, Garbage,
  data::Move,
  queue::{Bag, Queue},
};
use keyfinder::get_keys;
use search::{beam_search, eval::WEIGHTS_HANDTUNED};

pub struct StepResult {
  pub keys: Vec<Move>,
  pub time: f64,
}

pub struct Falcon {
  queue: Queue,
  game: Game,
  config: Option<GameConfig>,
}

impl Default for Falcon {
  fn default() -> Self {
    Self::new()
  }
}

impl Falcon {
  pub fn new() -> Self {
    let mut queue = Queue::new(Bag::Bag7, 0, 32, Vec::new());
    let game = Game::new(queue.shift(), queue.get_front_32());
    Self {
      queue,
      game,
      config: None,
    }
  }

  pub fn start(&mut self, config: GameConfig, seed: u64, bag: Bag) {
    self.queue = Queue::new(bag, seed, 32, Vec::new());
    self.config = Some(config);
    self.game = Game::new(self.queue.shift(), self.queue.get_front_32());
  }

  pub fn insert_garbage(&mut self, garbage: Vec<Garbage>) {
    for gb in garbage {
      self.game.board.insert_garbage(gb.amt, gb.col);
    }
    self.game.regen_collision_map();
  }

  pub fn step(&mut self, garbage: Vec<Garbage>) -> Option<StepResult> {
    let config = self.config.clone()?;
    self.game.garbage = garbage.into();

    let start_time = std::time::Instant::now();
    let choice = beam_search(self.game.clone(), &config, 5, &WEIGHTS_HANDTUNED);
    let elapsed = start_time.elapsed().as_secs_f64();

    if let Some(mv) = choice {
      let mut double_shift = false;
      if mv.0.3 {
        double_shift = self.game.hold.is_none();
        self.game.hold();
        self.game.regen_collision_map();
      }

      self.game.garbage.clear();

      let mut keys = get_keys(self.game.clone(), &config, (mv.0.0, mv.0.1, mv.0.2, mv.0.4));

      for key in keys.iter() {
        key.run(&mut self.game, &config);
      }

      if mv.0.3 {
        keys.insert(0, Move::Hold);
      }

      println!("-------------------------");
      self.game.print();
      println!("B2B: {}", self.game.b2b);
      println!("Time: {:.0}μs", elapsed * 1_000_000.0);

      self.game.hard_drop(&config);

      if double_shift {
        self.queue.shift();
      }
      self.queue.shift();
      self.game.queue = self.queue.get_front_32();
      self.game.queue_ptr = 0;

      self.game.regen_collision_map();

      Some(StepResult {
        keys,
        time: elapsed,
      })
    } else {
      self.game.hard_drop(&config);
      self.queue.shift();
      self.game.queue = self.queue.get_front_32();
      self.game.queue_ptr = 0;
      self.game.regen_collision_map();

      Some(StepResult {
        keys: vec![Move::HardDrop],
        time: elapsed,
      })
    }
  }
}

#[cfg(test)]
pub mod tests {
  use std::time::Instant;

  use super::*;
  use game::{BOARD_HEIGHT, BOARD_WIDTH, Game};
  use search::eval::WEIGHTS_HANDTUNED;
  use triangle::{
    engine::{queue::Mino, utils::KickTable},
    types::game::{ComboTable, Spin, SpinBonuses},
  };

  pub fn init() -> (game::GameConfig, Queue, Game) {
    let config = game::GameConfig {
      kicks: KickTable::SRSX,
      spins: SpinBonuses::Handheld,
      b2b_chaining: false,
      b2b_charging: true,
      b2b_charge_at: 0,
      b2b_charge_base: 0,
      pc_b2b: 1,
      pc_send: 5,
      combo_table: ComboTable::Multiplier,
      garbage_multiplier: 1.0,
      garbage_special_bonus: true,
    };

    let mut queue = Queue::new(Bag::Bag7, rand::random::<u64>(), 32, vec![Mino::Z]);

    let game = game::Game::new(queue.shift(), queue.get_front_32());

    (config, queue, game)
  }

  pub fn test_spins() {
    let (config, _, _) = init();

    let mut queue = Queue::new(Bag::Bag7, 0, 32, Vec::from([Mino::T]));

    let mut game = game::Game::new(queue.shift(), queue.get_front_32());
    println!("{}", game.piece.y);

    let points = [(0, 0), (1, 0), (2, 0)];

    for point in points.iter() {
      game.board.set(point.0, point.1);
    }

    game.regen_collision_map();

    println!("Initial board:");
    game.print();

    game.move_right();
    game.soft_drop();
    game.rotate(3, &config);

    println!("{:?}", game.spin);

    game.print();
  }

  pub fn test_game() {
    let (config, _, _) = init();

    let mut queue = Queue::new(Bag::Bag7, 0, 32, Vec::from([Mino::T]));

    let mut game = game::Game::new(queue.shift(), queue.get_front_32());

    let points = [
      (0, 0),
      (1, 0),
      (2, 0),
      (0, 1),
      (0, 2),
      (0, 3),
      (1, 3),
      (1, 4),
    ];

    for point in points.iter() {
      game.board.set(point.0, point.1);
    }
  }
}
