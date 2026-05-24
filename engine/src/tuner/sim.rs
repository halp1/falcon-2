use engine::game::queue::{Bag, Queue};
use engine::game::rng::RNG;
use engine::game::{BOARD_WIDTH, CollisionMap, Game, GameConfig, Garbage, StartState};
use engine::search::beam_search;
use engine::search::eval::Weights;
use rayon::prelude::*;
use triangle::types::game::Spin;

#[derive(PartialEq, Eq)]
pub enum MatchResult {
  WinA,
  WinB,
  Draw,
}

fn apply_move(
  game: &mut Game,
  mv: (u8, u8, u8, bool, Spin),
  config: &GameConfig,
  state: &StartState,
) -> (u16, Vec<Garbage>, bool, CollisionMap) {
  let double_shift = mv.3 && game.hold.is_none();
  if mv.3 {
    game.hold(state);
  }
  let map = game.collision_map();
  game.piece.x = mv.0;
  game.piece.y = mv.1;
  game.piece.rot = mv.2;
  game.spin = mv.4;
  let (sent, _) = game.hard_drop(config, &map, state, 0);

  game.queue_ptr = 0;

  let mut garbage = Vec::from(state.garbage);

  garbage = garbage.drain(game.garbage.0..).collect();
  if let Some(g) = garbage.first_mut() {
    g.amt = g.amt.saturating_sub(game.garbage.1);
  }

  (sent, garbage, double_shift, map)
}

pub fn run_match(
  weights_a: &Weights,
  weights_b: &Weights,
  seed: u64,
  config: &GameConfig,
  _depth: u8,
  max_moves: u32,
) -> MatchResult {
  let mut queue_a = Queue::<32>::new(Bag::Bag7, seed, vec![]);
  let mut queue_b = Queue::<32>::new(Bag::Bag7, seed, vec![]);
  let mut game_a = Game::new(queue_a.shift());
  let mut game_b = Game::new(queue_b.shift());
  let mut garbage_a: Vec<Garbage> = Vec::new();
  let mut garbage_b: Vec<Garbage> = Vec::new();
  let mut rng_a = RNG::new(seed ^ 0xDEAD_BEEF);
  let mut rng_b = RNG::new(seed ^ 0xCAFE_BABE);

  let start = std::time::Instant::now();

  for i in 0..max_moves {
    let (sent_a, remaining_a, double_shift_a, map_a) = {
      let queue_arr = queue_a.as_array();
      let state = StartState {
        garbage: garbage_a.as_slice(),
        queue: &queue_arr,
      };
      let mv = match beam_search::<7, 1000>(game_a.clone(), config, &state, weights_a) {
        None => return MatchResult::WinB,
        Some((mv, _)) => mv,
      };
      apply_move(&mut game_a, mv, config, &state)
    };
    garbage_a = remaining_a;
    for g in &mut garbage_a {
      assert!(g.time > 0, "garbage timer already 0");
      g.time -= 1;
    }
    if double_shift_a {
      queue_a.shift();
    }
    queue_a.shift();
    game_a.garbage = (0, 0);
    if game_a.topped_out(&map_a) {
      return MatchResult::WinB;
    }
    if sent_a > 0 {
      let col = (rng_a.next() % BOARD_WIDTH as u64) as u8;
      garbage_b.push(Garbage {
        col,
        amt: sent_a,
        time: 2,
      });
    }

    let (sent_b, remaining_b, double_shift_b, map_b) = {
      let queue_arr = queue_b.as_array();
      let state = StartState {
        garbage: garbage_b.as_slice(),
        queue: &queue_arr,
      };
      let mv = match beam_search::<7, 1000>(game_b.clone(), config, &state, weights_b) {
        None => return MatchResult::WinA,
        Some((mv, _)) => mv,
      };
      apply_move(&mut game_b, mv, config, &state)
    };
    garbage_b = remaining_b;
    for g in &mut garbage_b {
      assert!(g.time > 0, "garbage timer already 0");
      g.time -= 1;
    }
    if double_shift_b {
      queue_b.shift();
    }
    queue_b.shift();
    game_b.garbage = (0, 0);
    if game_b.topped_out(&map_b) {
      return MatchResult::WinA;
    }
    if sent_b > 0 {
      let col = (rng_b.next() % BOARD_WIDTH as u64) as u8;
      garbage_a.push(Garbage {
        col,
        amt: sent_b,
        time: 2,
      });
    }

    println!(
      "move {}, pace: {:.2}ms",
      i,
      start.elapsed().as_secs_f64() / (i + 1) as f64 * 1000.0
    );
  }

  MatchResult::Draw
}

// runs n parallel games, returns win-rate of weights_a (draws = 0.5)
pub fn run_batch(
  weights_a: &Weights,
  weights_b: &Weights,
  n: usize,
  base_seed: u64,
  config: &GameConfig,
  depth: u8,
  max_moves: u32,
) -> f64 {
  let score: usize = (0..n)
    .into_par_iter()
    .map(|i| {
      let seed = base_seed.wrapping_add(i as u64);
      match run_match(weights_a, weights_b, seed, config, depth, max_moves) {
        MatchResult::WinA => 2,
        MatchResult::WinB => 0,
        MatchResult::Draw => 1,
      }
    })
    .sum();
  score as f64 / (n * 2) as f64
}
