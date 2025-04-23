use std::collections::HashSet;

use crate::game::{Game, GameConfig, data::Move};

mod eval;
use eval::eval;

const MOVES: [Move; 6] = [
  Move::CW,
  Move::CCW,
  Move::Flip,
  Move::Left,
  Move::Right,
  Move::SoftDrop,
];

pub fn expand(
  mut state: &mut Game,
  passed: &mut [u64; 1024],
  res: &mut [(u8, u8, u8); 256],
) -> usize {
  for i in 0..1024 {
    passed[i] = 0;
  }

  let mut queue = [(0, 0, 0); 512];

  let mut front_ptr = 0;
  let mut back_ptr = 1;
  let mut res_ptr = 0;

  queue[back_ptr - 1] = (state.piece.x, state.piece.y, state.piece.rot);

  while front_ptr < back_ptr {
    let (x, y, rot) = queue[front_ptr];
    front_ptr += 1;

    for &mv in &MOVES {
      // Don't do these checks, because running the checks is more expensive the
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

      if !mv.run(&mut state) {
        continue;
      }

      let compressed = 0u16
        | (state.piece.x as u16 & 0b_1111)
        | ((state.piece.y as u16 & 0b_111111) << 4)
        | ((state.piece.rot as u16 & 0b11) << 10);

      let idx = compressed as usize / 64;
      let bit = 1 << (compressed % 64);

      if passed[idx] & bit != 0 {
        continue;
      }

      passed[idx] |= bit;

      if mv == Move::SoftDrop {
        res[res_ptr] = (state.piece.x, state.piece.y, state.piece.rot);
        res_ptr += 1;
      }

      queue[back_ptr] = (state.piece.x, state.piece.y, state.piece.rot);
      back_ptr += 1;
    }
  }

  res_ptr
}

#[derive(Clone, Debug)]
struct SearchState {
  pub game: Game,
  pub depth: u8,
  pub lines_sent: u16,
  pub first_move: Option<(u8, u8, u8, bool)>,
}

pub fn search(
  state: Game,
  config: &GameConfig,
  max_depth: u8,
) -> Option<((u8, u8, u8, bool), Game)> {
  let mut best_result: Option<(Game, f32, (u8, u8, u8, bool))> = None;

  let mut queue: Vec<SearchState> = Vec::with_capacity(2usize.pow(19));

  let mut passed: HashSet<[u64; 10]> = HashSet::with_capacity(2usize.pow(20));

  let mut expand_passed = [0u64; 65_536 / 64];
  let mut expand_res = [(0u8, 0u8, 0u8); 256];

  queue.push(SearchState {
    game: state,
    depth: 0,
    lines_sent: 0,
    first_move: None,
  });
  let mut ptr = 0;
  let mut nodes = 0u64;

  while ptr < queue.len() {
    let mut game_copy = queue[ptr].game.clone();
    let depth = queue[ptr].depth;
    let lines_sent = queue[ptr].lines_sent;
    let first_move = queue[ptr].first_move;
    ptr += 1;

    let moves = expand(&mut game_copy, &mut expand_passed, &mut expand_res);

    if depth >= max_depth - 1 {
      for i in 0..moves {
        let (x, y, rot) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        let lines = game_copy.hard_drop(config);
        nodes += 1;
        let score = eval(&game_copy, lines_sent + lines);
        if best_result.is_none() || score > best_result.as_ref().unwrap().1 {
          best_result = Some((game_copy.clone(), score, (x, y, rot, false)));
        }

        game_copy = queue[ptr - 1].game.clone();
      }
    } else {
      for i in 0..moves {
        let (x, y, rot) = expand_res[i];
        game_copy.piece.x = x;
        game_copy.piece.y = y;
        game_copy.piece.rot = rot;
        let lines = game_copy.hard_drop(config);
        if !passed.insert(game_copy.board.cols.clone()) {
          game_copy = queue[ptr - 1].game.clone();
          continue;
        }

        game_copy.regen_collision_map();
        nodes += 1;

        queue.push(SearchState {
          game: game_copy.clone(),
          depth: depth + 1,
          lines_sent: lines_sent + lines,
          first_move: if first_move != None {
            first_move
          } else {
            Some((x, y, rot, false))
          },
        });

        game_copy = queue[ptr - 1].game.clone();
      }
    }
  }

  println!("Total nodes evaluated: {}", nodes);

  if let Some(best) = best_result {
    Some((best.2, best.0))
  } else {
    None
  }
}
