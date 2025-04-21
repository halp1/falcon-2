use std::collections::{HashMap, HashSet};

use crate::game::{Game, GameConfig, data::Move};

mod eval;
use eval::eval;

use bitvec::prelude::*;

type BitSetU16 = BitArr!(for 65536, in u64, Msb0);

const MOVES: [Move; 6] = [
  Move::CW,
  Move::CCW,
  Move::Flip,
  Move::Left,
  Move::Right,
  Move::SoftDrop,
];

pub fn expand(mut state: &mut Game) -> Vec<(u8, u8, u8)> {
  let mut passed = BitSetU16::ZERO;
  let mut queue: Vec<(u8, u8, u8, Move)> = Vec::with_capacity(65_536);

  let mut ptr = 0;

  let mut res: Vec<(u8, u8, u8)> = Vec::with_capacity(256);

  queue.push((state.piece.x, state.piece.y, state.piece.rot, Move::None));
  while ptr < queue.len() {
    let (x, y, rot, last) = queue[ptr];
    ptr += 1;

    for &mv in &MOVES {
      if (last == Move::CCW && mv == Move::CW)
        || (last == Move::CW && mv == Move::CCW)
        || (last == Move::Flip && mv == Move::Flip)
        || (last == Move::Left && mv == Move::Right)
        || (last == Move::Right && mv == Move::Left)
        || (last == Move::SoftDrop && mv == Move::SoftDrop)
      {
        continue;
      }
      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;

      mv.run(&mut state);

      let compressed = 0u16
        | (state.piece.x as u16 & 0b_1111)
        | ((state.piece.y as u16 & 0b_111111) << 4)
        | ((state.piece.rot as u16 & 0b11) << 10);

      if passed[compressed as usize] {
        continue;
      }

      if mv == Move::SoftDrop {
        res.push((state.piece.x, state.piece.y, state.piece.rot));
      }

      passed.set(compressed as usize, true);

      queue.push((state.piece.x, state.piece.y, state.piece.rot, mv));
    }
  }

  res
}

#[derive(Clone, Debug)]
struct SearchState {
  pub game: Game,
  pub depth: u8,
  pub lines_sent: u16,
  pub first_move: Option<(u8, u8, u8, bool)>,
}

pub fn search(state: Game, config: &GameConfig, max_depth: u8) -> Option<(u8, u8, u8, bool)> {
  let mut best_result: Option<(Game, f32, (u8, u8, u8, bool))> = None;

  let mut queue: Vec<SearchState> = Vec::with_capacity(2usize.pow(19));

  let mut passed: HashSet<[u64; 10]> = HashSet::with_capacity(2usize.pow(20));

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

    let moves = expand(&mut game_copy);

    if depth >= max_depth - 1 {
      for (x, y, rot) in moves {
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
      for (x, y, rot) in moves {
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
        let new_first_move = first_move.or(Some((x, y, rot, false)));

        queue.push(SearchState {
          game: game_copy.clone(),
          depth: depth + 1,
          lines_sent: lines_sent + lines,
          first_move: new_first_move,
        });

        game_copy = queue[ptr - 1].game.clone();
      }
    }
  }

  println!("Total nodes evaluated: {}", nodes);

  if let Some(best) = best_result {
    best.0.board.print();
    Some(best.2)
  } else {
    None
  }
}
