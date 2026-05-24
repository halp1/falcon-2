use engine::game::queue::{Bag, Queue};
use engine::game::rng::RNG;
use engine::game::{BOARD_WIDTH, CollisionMap, Game, GameConfig, Garbage, StartState};
use engine::search::beam_search;
use engine::search::eval::Weights;
use rayon::prelude::*;
use triangle::types::game::Spin;

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
) -> f64 {
  let mut queue_a = Queue::<32>::new(Bag::Bag7, seed, vec![]);
  let mut queue_b = Queue::<32>::new(Bag::Bag7, seed, vec![]);
  let mut game_a = Game::new(queue_a.shift());
  let mut game_b = Game::new(queue_b.shift());
  let mut garbage_a: Vec<Garbage> = Vec::new();
  let mut garbage_b: Vec<Garbage> = Vec::new();
  let mut rng_a = RNG::new(seed ^ 0xDEAD_BEEF);
  let mut rng_b = RNG::new(seed ^ 0xCAFE_BABE);
  let mut sent_a_total: u32 = 0;
  let mut sent_b_total: u32 = 0;

  for _ in 0..max_moves {
    let (sent_a, remaining_a, double_shift_a, map_a) = {
      let queue_arr = queue_a.as_array();
      let state = StartState {
        garbage: garbage_a.as_slice(),
        queue: &queue_arr,
      };
      let mv = match beam_search::<7, 60>(game_a.clone(), config, &state, weights_a) {
        None => return 0.0,
        Some((mv, _)) => mv,
      };
      apply_move(&mut game_a, mv, config, &state)
    };
    garbage_a = remaining_a;
    for g in &mut garbage_a {
      g.time = g.time.saturating_sub(1);
    }
    if double_shift_a {
      queue_a.shift();
    }
    queue_a.shift();
    game_a.garbage = (0, 0);
    if game_a.topped_out(&map_a) {
      return 0.0;
    }
    if sent_a > 0 {
      sent_a_total += sent_a as u32;
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
      let mv = match beam_search::<7, 60>(game_b.clone(), config, &state, weights_b) {
        None => return 1.0,
        Some((mv, _)) => mv,
      };
      apply_move(&mut game_b, mv, config, &state)
    };
    garbage_b = remaining_b;
    for g in &mut garbage_b {
      g.time = g.time.saturating_sub(1);
    }
    if double_shift_b {
      queue_b.shift();
    }
    queue_b.shift();
    game_b.garbage = (0, 0);
    if game_b.topped_out(&map_b) {
      return 1.0;
    }
    if sent_b > 0 {
      sent_b_total += sent_b as u32;
      let col = (rng_b.next() % BOARD_WIDTH as u64) as u8;
      garbage_a.push(Garbage {
        col,
        amt: sent_b,
        time: 2,
      });
    }
  }

  // neither topped out: score by garbage-sent ratio (laplace smoothing avoids 0/0)
  (sent_a_total as f64 + 1.0) / (sent_a_total as f64 + sent_b_total as f64 + 2.0)
}

// runs n parallel games, returns average score for weights_a in [0, 1]
pub fn run_batch(
  weights_a: &Weights,
  weights_b: &Weights,
  n: usize,
  base_seed: u64,
  config: &GameConfig,
  depth: u8,
  max_moves: u32,
) -> f64 {
  let total: f64 = (0..n)
    .into_par_iter()
    .map(|i| {
      run_match(
        weights_a,
        weights_b,
        base_seed.wrapping_add(i as u64),
        config,
        depth,
        max_moves,
      )
    })
    .sum();
  total / n as f64
}
