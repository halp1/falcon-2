#![allow(unused_variables)]

use std::{collections::HashSet, time::Instant};

use crate::game::StartState;
use crate::game::{BOARD_WIDTH, Game, GameConfig};
use crate::search::eval::MoveInfo;
use crate::search::movegen::expand;

pub mod eval;
pub mod movegen;
use eval::Weights;
use triangle::types::game::Spin;

#[derive(Clone, Debug)]
struct SearchState {
  pub game: Game,
  pub depth: u8,
  pub first_move: Option<(u8, u8, u8, bool, Spin)>,
}

pub fn search(
  state: Game,
  config: &GameConfig,
  start_state: &StartState,
  max_depth: u8,
  weights: &Weights,
) -> Option<((u8, u8, u8, bool, Spin), Game)> {
  let mut best_result: Option<(Game, f64, (u8, u8, u8, bool, Spin))> = None;

  let mut queue: Vec<SearchState> = Vec::with_capacity(2usize.pow(19));

  let mut passed: HashSet<[u64; BOARD_WIDTH]> = HashSet::with_capacity(2usize.pow(20));

  let mut expand_passed = [0u64; 2048];
  let mut expand_res = [(0u8, 0u8, 0u8, Spin::None); 512];

  queue.push(SearchState {
    game: state,
    depth: 0,
    first_move: None,
  });
  let mut ptr = 0;

  let mut nodes = 0u64;
  let start = Instant::now();

  while ptr < queue.len() {
    let mut game_copy = queue[ptr].game.clone();
    let depth = queue[ptr].depth;
    let first_move = queue[ptr].first_move;
    ptr += 1;

    let map = game_copy.collision_map();

    let moves = expand(
      &mut game_copy,
      config,
      &map,
      &start_state,
      &mut expand_passed,
      &mut expand_res,
    );

    if depth >= max_depth - 1 {
      for i in 0..moves.0 {
        let (x, y, rot, spin) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        game_copy.spin = spin;
        let (sent, clear) = game_copy.hard_drop(config, &map, &start_state, depth);
        nodes += 1;
        if game_copy.topped_out(&map) {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }
        let score = weights.eval(&game_copy, &MoveInfo { clear, sent });
        if best_result.is_none() || score > best_result.as_ref().unwrap().1 {
          best_result = Some((
            game_copy.clone(),
            score,
            first_move.unwrap_or((x, y, rot, false, spin)),
          ));
        }

        game_copy = queue[ptr - 1].game.clone();
      }
    } else {
      for i in 0..moves.0 {
        let (x, y, rot, spin) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        game_copy.spin = spin;
        let (sent, clear) = game_copy.hard_drop(config, &map, &start_state, depth);
        if !passed.insert(game_copy.board.cols.clone()) {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        nodes += 1;

        if game_copy.topped_out_raw() {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        queue.push(SearchState {
          game: game_copy.clone(),
          depth: depth + 1,
          first_move: if first_move != None {
            first_move
          } else {
            Some((x, y, rot, false, spin))
          },
        });

        game_copy = queue[ptr - 1].game.clone();
      }
    }
  }

  let elapsed = start.elapsed();

  println!("Total nodes evaluated: {}", nodes);
  println!("NPS: {}", nodes as f32 / elapsed.as_secs_f32());

  if let Some(best) = best_result {
    Some((best.2, best.0))
  } else {
    None
  }
}

// beam search 🥶

use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone)]
struct Candidate {
  state: SearchState,
  score: f64,
}

impl PartialEq for Candidate {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score
  }
}
impl PartialOrd for Candidate {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.score.total_cmp(&other.score))
  }
}

impl Eq for Candidate {}

impl Ord for Candidate {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.partial_cmp(other).unwrap()
  }
}

pub fn beam_search<const DEPTH: u8, const WIDTH: usize>(
  root_game: Game,
  config: &GameConfig,
  start_state: &StartState,
  weights: &Weights,
) -> Option<((u8, u8, u8, bool, Spin), Game)> {
  let init_state = SearchState {
    game: root_game.clone(),
    depth: 0,
    first_move: None,
  };
  let init_score = weights.eval(
    &root_game,
    &MoveInfo {
      clear: (Spin::None, 0),
      sent: 0,
    },
  );

  let mut beam: BinaryHeap<Reverse<Candidate>> = BinaryHeap::with_capacity(WIDTH);
  beam.push(Reverse(Candidate {
    state: init_state,
    score: init_score,
  }));

  let mut passed = [0u64; 2048];
  let mut res_buf = [(0u8, 0u8, 0u8, Spin::None); 512];

  for depth in 0..DEPTH {
    let mut next_beam: BinaryHeap<Reverse<Candidate>> = BinaryHeap::with_capacity(WIDTH);

    while let Some(Reverse(cand)) = beam.pop() {
      for n in 0..=1 {
        let mut game_copy = cand.state.game.clone();

        if n == 1 {
          game_copy.hold(&start_state);
        }

        let map = game_copy.collision_map();

        let moves = expand(
          &mut game_copy,
          config,
          &map,
          &start_state,
          &mut passed,
          &mut res_buf,
        );

        for i in 0..moves.0 {
          let (x, y, rot, spin) = res_buf[i];
          let mut g2 = game_copy.clone();
          g2.piece.x = x;
          g2.piece.y = y;
          g2.piece.rot = rot;
          g2.spin = spin;

          let (sent, clear) = g2.hard_drop(config, &map, &start_state, depth);

          if g2.topped_out_raw() {
            continue;
          }

          let next_depth = cand.state.depth + 1;
          let next_first = cand.state.first_move.or(Some((x, y, rot, n == 1, spin)));

          let next_state = SearchState {
            game: g2.clone(),
            depth: next_depth,
            first_move: next_first,
          };

          let score = weights.eval(&g2, &MoveInfo { clear, sent });
          let candidate = Candidate {
            state: next_state,
            score,
          };

          if next_beam.len() < WIDTH {
            next_beam.push(Reverse(candidate));
          } else if let Some(Reverse(worst)) = next_beam.peek() {
            if score > worst.score {
              next_beam.pop();
              next_beam.push(Reverse(candidate));
            }
          }
        }
      }
    }

    if next_beam.is_empty() {
      break;
    }
    beam = next_beam;
  }

  beam
    .into_iter()
    .map(|rev| rev.0)
    .max_by(|a, b| {
      a.score
        .partial_cmp(&b.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    })
    .and_then(|cand| cand.state.first_move.map(|m| (m, cand.state.game)))
}
