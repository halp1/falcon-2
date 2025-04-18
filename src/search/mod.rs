use std::collections::VecDeque;

use crate::game::{Game, GameConfig, data::Move};

mod compressor;
use compressor::{
  compress_key, compress_move, convert_compressed, decompress_key, decompress_move,
};

mod eval;
use eval::eval;

use bitvec::prelude::*;

type BitSetU16 = BitArr!(for 65536, in u64, Msb0);

pub fn expand(state: Game, config: &GameConfig) -> Vec<(Game, u16, u16)> {
  let mut state = state.clone();

  let mut passed = BitSetU16::ZERO;

  let mut queue: VecDeque<u16> = VecDeque::new();

  queue.push_back(compress_key(&state, Move::None));

  let mut results: Vec<(Game, u16)> = Vec::new();

  while let Some(pos) = queue.pop_front() {
    let (x, y, rot, last) = decompress_key(pos);

    let mut options: VecDeque<Move> = VecDeque::new();

    if last != Move::CCW {
      options.push_back(Move::CW);
    }
    if last != Move::CW {
      options.push_back(Move::CCW);
    }
    if last != Move::Flip {
      options.push_back(Move::Flip);
    }
    if last != Move::Left {
      options.push_back(Move::Right);
    }
    if last != Move::Right {
      options.push_back(Move::Left);
    }
    if last != Move::SoftDrop {
      options.push_back(Move::SoftDrop);
    }

    while let Some(mv) = options.pop_front() {
      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;

      mv.run(&mut state);

      let compressed = compress_key(&state, mv);

      if passed[compressed as usize] {
        continue;
      }

      if mv == Move::SoftDrop {
        results.push((state.clone(), compressed));
      }

      passed.set(compressed as usize, true);

      queue.push_back(compressed);
    }
  }

  let mut final_results: Vec<(Game, u16, u16)> = Vec::new();

  for (game, mv) in results {
    let mut game_copy = game;
    let lines = game_copy.hard_drop(config);
    game_copy.regen_collision_map();
    if !game_copy.topped_out() {
      final_results.push((game_copy, mv, lines));
    }
  }

  final_results
}

struct SearchState {
  pub game: Game,
  pub depth: u8,
  pub lines_sent: u16,
  pub first_move: Option<u16>,
}

pub fn search(state: Game, config: &GameConfig, max_depth: u8) -> Option<(u8, u8, u8, bool)> {
  let mut best_result: Option<(Game, f32, u16)> = None;
  let mut queue: VecDeque<SearchState> = VecDeque::new();

  queue.push_back(SearchState {
    game: state,
    depth: 0,
    lines_sent: 0,
    first_move: None,
  });

  while let Some(search_state) = queue.pop_front() {
    let moves = expand(search_state.game.clone(), config);
    if search_state.depth >= max_depth - 1 {
      for (game, _, lines) in moves {
        let score = eval(&game, search_state.lines_sent + lines);
        if best_result.is_none() || score > best_result.as_ref().unwrap().1 {
          best_result = Some((
            game,
            score,
            convert_compressed(search_state.first_move.unwrap_or(0), false),
          ));
        }
      }
    } else {
      for (game, mv, lines) in moves {
        let new_state = SearchState {
          game,
          depth: search_state.depth + 1,
          lines_sent: search_state.lines_sent + lines,
          first_move: search_state.first_move.or(Some(mv)),
        };

        queue.push_back(new_state);
      }
    }
  }

  if let Some(best) = best_result {
		best.0.board.print();
    Some(decompress_move(best.2))
  } else {
    None
  }
}
