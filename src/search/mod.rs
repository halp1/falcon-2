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

    let mut softdrops: Vec<(u8, u8, u8)> = Vec::with_capacity(256);

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
                | ((state.piece.rot as u16 & 0b11) << 10)
                | (mv as u16 & 0b1111) << 12;

            if passed[compressed as usize] {
                continue;
            }

            if mv == Move::SoftDrop {
                softdrops.push((state.piece.x, state.piece.y, state.piece.rot));
            }

            passed.set(compressed as usize, true);

            queue.push((state.piece.x, state.piece.y, state.piece.rot, mv));
        }
    }

    softdrops
}

#[derive(Clone, Copy, Debug)]
struct SearchState {
    pub game: Game,
    pub depth: u8,
    pub lines_sent: u16,
    pub first_move: Option<(u8, u8, u8, bool)>,
}

pub fn search(state: Game, config: &GameConfig, max_depth: u8) -> Option<(u8, u8, u8, bool)> {
    let mut best_result: Option<(Game, f32, (u8, u8, u8, bool))> = None;

    let mut queue: Vec<SearchState> = Vec::with_capacity(65_536);

    queue.push(SearchState {
        game: state,
        depth: 0,
        lines_sent: 0,
        first_move: None,
    });

    let mut ptr = 0;

    let mut nodes = 0u64;

    while ptr < queue.len() {
        let mut search_state = queue[ptr];
        ptr += 1;

        let moves = expand(&mut search_state.game);
        if search_state.depth >= max_depth - 1 {
            for (x, y, rot) in moves {
                search_state.game.piece.x = x;
                search_state.game.piece.y = y;
                search_state.game.piece.rot = rot;
                let lines = search_state.game.hard_drop(config);
                nodes += 1;
                let score = eval(&search_state.game, search_state.lines_sent + lines);
                if best_result.is_none() || score > best_result.as_ref().unwrap().1 {
                    best_result = Some((search_state.game, score, (x, y, rot, false)));
                }
            }
        } else {
            for (x, y, rot) in moves {
								search_state.game.piece.x = x;
								search_state.game.piece.y = y;
								search_state.game.piece.rot = rot;
								let lines = search_state.game.hard_drop(config);
                nodes += 1;
                let new_state = SearchState {
                    game: search_state.game.clone(),
                    depth: search_state.depth + 1,
                    lines_sent: search_state.lines_sent + lines,
                    first_move: search_state.first_move.or(Some((x, y, rot, false))),
                };

                queue.push(new_state);
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
