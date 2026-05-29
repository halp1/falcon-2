use engine::{
  game::{BOARD_WIDTH, Game, GameConfig, Garbage, StartState, queue::Queue, rng::RNG},
  search::{Action, beam_search, eval::Weights},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn apply_move(
  game: &mut Game,
  mv: Action,
  config: &GameConfig,
  state: &StartState,
) -> (u16, u16, Vec<Garbage>, bool) {
  let double_shift = mv.hold && game.hold.is_none();
  if mv.hold {
    game.hold(state);
  }
  let map = game.collision_map();
  game.piece.x = mv.placement.x;
  game.piece.y = mv.placement.y;
  game.piece.rot = mv.placement.rot;
  game.spin = mv.placement.spin;
  let (attack, sent, _) = game.hard_drop(config, &map, state, 0);

  game.queue_ptr = 0;

  let mut garbage = Vec::from(state.garbage);

  garbage = garbage.drain(game.garbage.0..).collect();
  if let Some(g) = garbage.first_mut() {
    g.amt = g.amt.saturating_sub(game.garbage.1);
  }

  while garbage.first().map_or(false, |g| g.amt == 0) {
    garbage.remove(0);
  }

  (attack, sent, garbage, double_shift)
}

struct Player {
  weights: Weights,
  queue: Queue<32>,
  game: Game,
  garbage: Vec<Garbage>,
  rng: RNG,
  sent_total: u32,
}

/// returns true if b wins, false if a wins, a wins by default on ties
pub fn run_match<const DEPTH: u8, const WIDTH: usize>(
  config: &GameConfig,
  max_moves: usize,
  weights_a: &Weights,
  weights_b: &Weights,
  seed: u64,
) -> bool {
  let mut players = (0..2)
    .map(|i| {
      let mut queue = Queue::new(config.bag, seed, vec![]);
      Player {
        weights: if i == 0 {
          weights_a.clone()
        } else {
          weights_b.clone()
        },
        game: Game::new(queue.shift()),
        queue,
        garbage: Vec::new(),
        rng: RNG::new(seed),
        sent_total: 0,
      }
    })
    .collect::<Vec<_>>();

  for _ in 0..max_moves {
    let opponent_games: Vec<Game> = players.iter().map(|p| p.game.clone()).collect();

    let results: Vec<(bool, usize, u16)> = players
      .iter_mut()
      .enumerate()
      .map(|(i, player)| -> (bool, usize, u16) {
        let arr = player.queue.as_array();
        let state = StartState {
          garbage: player.garbage.as_slice(),
          queue: &arr,
        };

        let gc = player.game.clone();

        let (attack, sent, garbage, double_shift) = apply_move(
          &mut player.game,
          match beam_search::<DEPTH, WIDTH>(
            gc,
            config,
            &state,
            &player.weights,
            player.weights.eval_opponent(&opponent_games[(i + 1) % 2]),
          ) {
            Some(mv) => mv.0,
            None => return (false, i, 0),
          },
          config,
          &state,
        );

        if player.game.topped_out_raw() {
          return (false, i, 0);
        }

        player.sent_total += attack as u32;
        player.garbage = garbage;

        if double_shift {
          player.queue.shift();
        }
        player.queue.shift();
        player.game.queue_ptr = 0;
        player.game.garbage = (0, 0);

        (true, i, sent)
      })
      .collect();

    for &(_, i, sent) in &results {
      if sent > 0 {
        let opponent = &mut players[1 - i];
        opponent.garbage.push(Garbage {
          amt: (sent as f32 * config.garbage_multiplier).floor() as u16,
          col: (opponent.rng.next_float() * BOARD_WIDTH as f64) as u8,
          time: 2,
        });
      }
    }

    if !results[1].0 {
      return false;
    }
    if !results[0].0 {
      return true;
    }
  }

  return players[0].sent_total < players[1].sent_total;
}

pub fn batch_match<const DEPTH: u8, const WIDTH: usize>(
  weights_a: &Weights,
  weights_b: &Weights,
  n: usize,
  config: &GameConfig,
  max_moves: usize,
  seed: u64,
) -> f64 {
  let total = (0..n)
    .into_par_iter()
    .map(|i| {
      run_match::<DEPTH, WIDTH>(
        config,
        max_moves,
        weights_a,
        weights_b,
        seed.wrapping_add(i as u64),
      )
    })
    .map(|b| if b { 0.0 } else { 1.0 })
    .sum::<f64>();
  total / n as f64
}

pub fn run_solo<const DEPTH: u8, const WIDTH: usize>(
  weights: &Weights,
  config: &GameConfig,
  moves: usize,
  garbage_frequency: usize,
  seed: u64,
) -> f64 {
  let mut player = {
    let mut queue = Queue::new(config.bag, seed, vec![]);
    Player {
      weights: weights.clone(),
      game: Game::new(queue.shift()),
      queue,
      garbage: Vec::new(),
      rng: RNG::new(seed ^ 0x9e3779b97f4a7c15),
      sent_total: 0,
    }
  };

  for i in 0..moves {
    let arr = player.queue.as_array();
    let state = StartState {
      garbage: player.garbage.as_slice(),
      queue: &arr,
    };

    let gc = player.game.clone();

    let (attack, _, garbage, double_shift) = apply_move(
      &mut player.game,
      match beam_search::<DEPTH, WIDTH>(gc, config, &state, &player.weights, 0.0) {
        Some(mv) => mv.0,
        None => return i as f64 + player.sent_total as f64,
      },
      config,
      &state,
    );

    if player.game.topped_out_raw() {
      return i as f64 + player.sent_total as f64 + attack as f64;
    }

    player.sent_total += attack as u32;

    if double_shift {
      player.queue.shift();
    }
    player.queue.shift();
    player.game.queue_ptr = 0;
    player.game.garbage = (0, 0);

    player.garbage = garbage
      .into_iter()
      .map(|mut g| {
        g.time = g.time.saturating_sub(1);
        g
      })
      .collect();

    if i % garbage_frequency == 0 {
      player.garbage.push(Garbage {
        amt: (player.rng.next_float() * 8.0 + 1.0).floor() as u16,
        col: (player.rng.next_float() * BOARD_WIDTH as f64) as u8,
        time: if player.rng.next_float() < 1.0 / 3.0 {
          1
        } else {
          0
        }, // average 60 frames/piece and 20 frames of garbage delay so 1/3 chance of time 1, otherwise time 0
      });
    }
  }

  moves as f64 + player.sent_total as f64
}

pub fn batch_solo<const DEPTH: u8, const WIDTH: usize>(
  weights: &Weights,
  config: &GameConfig,
  moves: usize,
  garbage_frequency: usize,
  n: usize,
  seed: u64,
) -> f64 {
  let total = (0..n)
    .into_par_iter()
    .map(|i| {
      run_solo::<DEPTH, WIDTH>(
        weights,
        config,
        moves,
        garbage_frequency,
        seed.wrapping_add(i as u64),
      )
    })
    .sum::<f64>();
  total / n as f64
}
