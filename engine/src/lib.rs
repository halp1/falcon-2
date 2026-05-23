pub mod game;
pub mod io;
pub mod keyfinder;
pub mod search;

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
