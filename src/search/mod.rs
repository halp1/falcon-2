#![allow(unused_variables)]

use std::{collections::HashSet, time::Instant};

use crate::game::{
  self, BOARD_WIDTH, Game, GameConfig,
  data::{Mino, Move, Spin},
};

pub mod eval;
use eval::eval;

const MOVES: [[Move; 6]; 7] = [
  // None
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
  ],
  // Left
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::SoftDrop,
    Move::None,
  ],
  // Right
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
  // Softdrop
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::None,
  ],
  // CCW
  [
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
  // CW
  [
    Move::CW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
  // Flip
  [
    Move::CW,
    Move::CCW,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::None,
  ],
];

pub fn expand(
  mut state: &mut Game,
  config: &GameConfig,
  passed: &mut [u64; 2048],
  res: &mut [(u8, u8, u8, Spin); 512],
) -> (usize, u64) {
  passed.iter_mut().for_each(|m| *m = 0);

  let mut queue = [(0, 0, 0, Spin::None, Move::None); 512];

  let mut front_ptr = 0;
  let mut back_ptr = 1;
  let mut res_ptr = 0;

  let mut nodes = 0u64;

  queue[0] = (
    state.piece.x,
    state.piece.y,
    state.piece.rot,
    Spin::None,
    Move::None,
  );

  while front_ptr < back_ptr {
    let (x, y, rot, spin, prev_mv) = queue[front_ptr];
    front_ptr += 1;

    for &mv in &MOVES[prev_mv as usize] {
      nodes += 1;
      if mv == Move::None {
        break;
      }
      // Don't do these checks, because running the checks is more expensive
      // if (last == Move::CCW && mv == Move::CW)
      //     || (last == Move::CW && mv == Move::CCW)
      //     || (last == Move::Flip && mv == Move::Flip)
      //     || (last == Move::Left && mv == Move::Right)
      //     || (last == Move::Right && mv == Move::Left)
      //     || (last == Move::SoftDrop && mv == Move::SoftDrop)
      // {
      //     continue;
      // }

      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;
      state.spin = spin;

      let fail = !mv.run(&mut state, config);

      let mut compressed =
        0u16 | (state.piece.x as u16 & 0b_1111) | ((state.piece.y as u16 & 0b_111111) << 4);

      if state.piece.mino != Mino::O {
        compressed |= ((state.piece.rot as u16 & 0b11) << 10) | ((state.spin as u16 & 0b11) << 12);
      }

      let idx = compressed as usize / 64;
      let bit = 1 << (compressed % 64);

      if mv == Move::SoftDrop && passed[1024 + idx] & bit == 0 {
        passed[1024 + idx] |= bit;
        res[res_ptr] = (state.piece.x, state.piece.y, state.piece.rot, state.spin);
        res_ptr += 1;
      }

      if fail || passed[idx] & bit != 0 {
        continue;
      }

      passed[idx] |= bit;

      queue[back_ptr] = (
        state.piece.x,
        state.piece.y,
        state.piece.rot,
        state.spin,
        mv,
      );
      back_ptr += 1;
    }
  }

  (res_ptr, nodes)
}

#[derive(Clone, Debug)]
struct SearchState {
  pub game: Game,
  pub depth: u8,
  pub lines_sent: u16,
  pub clears: Vec<Spin>,
  pub first_move: Option<(u8, u8, u8, bool, Spin)>,
}

pub fn search(
  state: Game,
  config: &GameConfig,
  max_depth: u8,
) -> Option<((u8, u8, u8, bool, Spin), Game)> {
  let mut best_result: Option<(Game, i32, (u8, u8, u8, bool, Spin))> = None;

  let mut queue: Vec<SearchState> = Vec::with_capacity(2usize.pow(19));

  let mut passed: HashSet<[u64; BOARD_WIDTH]> = HashSet::with_capacity(2usize.pow(20));

  let mut expand_passed = [0u64; 2048];
  let mut expand_res = [(0u8, 0u8, 0u8, Spin::None); 512];

  queue.push(SearchState {
    game: state,
    depth: 0,
    lines_sent: 0,
    clears: Vec::with_capacity(16),
    first_move: None,
  });
  let mut ptr = 0;

  let mut nodes = 0u64;
  let start = Instant::now();

  while ptr < queue.len() {
    let mut game_copy = queue[ptr].game.clone();
    let depth = queue[ptr].depth;
    let lines_sent = queue[ptr].lines_sent;
    let first_move = queue[ptr].first_move;
    let mut clears = queue[ptr].clears.clone();
    ptr += 1;

    let moves = expand(&mut game_copy, config, &mut expand_passed, &mut expand_res);

    if depth >= max_depth - 1 {
      for i in 0..moves.0 {
        let (x, y, rot, spin) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        game_copy.spin = spin;
        let (lines, clear) = game_copy.hard_drop(config);
        if let Some(c) = clear {
          clears.push(c);
        }
        nodes += 1;
        if game_copy.topped_out() {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }
        let score = eval(&game_copy, lines_sent + lines, clears.clone());
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
        let (lines, clear) = game_copy.hard_drop(config);
        if !passed.insert(game_copy.board.cols.clone()) {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        game_copy.regen_collision_map();

        nodes += 1;

        if game_copy.topped_out() {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        if let Some(c) = clear {
          clears.push(c);
        }

        queue.push(SearchState {
          game: game_copy.clone(),
          depth: depth + 1,
          lines_sent: lines_sent + lines,
          clears: clears.clone(),
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
