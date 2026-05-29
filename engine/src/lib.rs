#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub mod game;
// pub mod game2;
pub mod io;
pub mod keyfinder;
pub mod search;

use game::{
  Game, GameConfig, Garbage, StartState,
  data::Move,
  queue::{Bag, Queue},
};
use keyfinder::get_keys;
use search::beam_search;

use crate::search::eval::Weights;

pub struct StepResult {
  pub keys: Vec<Move>,
  pub time: f64,
}

pub struct Falcon<const DEPTH: u8, const WIDTH: usize> {
  queue: Queue<32>,
  game: Game,
  config: Option<GameConfig>,
  weights: Weights,
}

impl<const DEPTH: u8, const WIDTH: usize> Falcon<DEPTH, WIDTH> {
  pub fn new(weights: Weights) -> Self {
    let mut queue = Queue::new(Bag::Bag7, 0, Vec::new());
    let game = Game::new(queue.shift());

    Self {
      queue,
      game,
      config: None,
      weights,
    }
  }

  pub fn start(&mut self, config: GameConfig, seed: u64, bag: Bag) {
    self.queue = Queue::new(bag, seed, Vec::new());
    self.config = Some(config);
    self.game = Game::new(self.queue.shift());
  }

  pub fn insert_garbage(&mut self, garbage: Vec<Garbage>) {
    for gb in garbage {
      self.game.board.insert_garbage(gb.amt, gb.col);
    }
  }

  pub fn step(&mut self, garbage: Vec<Garbage>, opponent: &Game) -> Option<StepResult> {
    let config = self.config.clone()?;
    self.game.garbage = (0, 0);

    let queue_arr = self.queue.as_array();
    let start_state = StartState {
      queue: &queue_arr,
      garbage: garbage.as_slice(),
    };

    let start_time = std::time::Instant::now();
    let choice = beam_search::<DEPTH, WIDTH>(
      self.game.clone(),
      &config,
      &start_state,
      &self.weights,
      self.weights.eval_opponent(opponent),
    );
    let elapsed = start_time.elapsed().as_secs_f64();

    if let Some(mv) = choice {
      let mut double_shift = false;
      if mv.0.hold {
        double_shift = self.game.hold.is_none();
        self.game.hold(&start_state);
      }

      let map = self.game.collision_map();

      let mut keys = get_keys(self.game.clone(), &config, mv.0.placement);

      for key in keys.iter() {
        key.run(&mut self.game, &config, &map, &start_state);
      }

      if mv.0.hold {
        keys.insert(0, Move::Hold);
      }

      println!("-------------------------");
      self.game.print();
      println!("B2B: {}", self.game.b2b);
      println!("Time: {:.0}μs", elapsed * 1_000_000.0);

      self.game.hard_drop(
        &config,
        &map,
        &StartState {
          queue: &self.queue.as_array(),
          garbage: &[],
        },
        0,
      );

      if double_shift {
        self.queue.shift();
      }
      self.queue.shift();
      self.game.queue_ptr = 0;

      Some(StepResult {
        keys,
        time: elapsed,
      })
    } else {
      let map = self.game.collision_map();
      self.game.hard_drop(&config, &map, &start_state, 0);
      self.queue.shift();
      self.game.queue_ptr = 0;

      Some(StepResult {
        keys: vec![Move::HardDrop],
        time: elapsed,
      })
    }
  }
}

#[cfg(test)]
pub mod tests {

  use super::*;
  use game::Game;
  use triangle::{
    engine::{queue::Mino, utils::KickTable},
    types::game::{ComboTable, SpinBonuses},
  };

  pub fn init() -> (game::GameConfig, Queue<32>, Game) {
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
      garbage_cap: 8,
      garbage_special_bonus: true,
      bag: Bag::Bag7,
    };

    let mut queue = Queue::<32>::new(Bag::Bag7, rand::random::<u64>(), vec![Mino::Z]);

    let game = game::Game::new(queue.shift());

    (config, queue, game)
  }

  pub fn test_spins() {
    let (config, _, _) = init();

    let mut queue = Queue::<32>::new(Bag::Bag7, 0, vec![Mino::T]);

    let mut game = game::Game::new(queue.shift());
    println!("{}", game.piece.y);

    let points = [(0, 0), (1, 0), (2, 0)];

    for point in points.iter() {
      game.board.set(point.0, point.1);
    }

    let map = game.collision_map();

    println!("Initial board:");
    game.print();

    game.move_right(&map);
    game.soft_drop(&map);
    game.rotate(3, &config, &map);

    println!("{:?}", game.spin);

    game.print();
  }

  #[test]
  pub fn test_game() {
    // let (config, _, _) = init();

    let mut queue = Queue::<32>::new(Bag::Bag7, 0, vec![Mino::T]);

    let mut game = game::Game::new(queue.shift());

    let points = [
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      // (6, 0),
      (7, 0),
      (8, 0),
      (9, 0),
      (0, 1),
      (1, 1),
      (2, 1),
      // (3, 1),
      (4, 1),
      (5, 1),
      (6, 1),
      // (7, 1),
      // (8, 1),
      (9, 1),
      (0, 2),
      (1, 2),
      (2, 2),
      (3, 2),
      (4, 2),
      (5, 2),
      (6, 2),
      (7, 2),
      // (8, 2),
      (9, 2),
    ];

    for point in points.iter() {
      game.board.set(point.0, point.1);
    }

    game.print();

    let heights = &game.board.column_heights();
    println!("heights: {:?}", heights);
    println!("split heights: {:?}", game.board.heights());
    println!("well: {:?}", game.board.well(heights));

    println!("holes: {:?}", game.board.holes(heights));
  }
}
