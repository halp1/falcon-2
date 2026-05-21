use triangle::types::game::Spin;

use crate::engine::game::data::MinoData;

use super::game::{Game, GameConfig, data::Move};

const MOVES: [[Move; 9]; 9] = [
  // None
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::DasLeft,
    Move::DasRight,
    Move::HardDrop,
  ],
  // Left
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::SoftDrop,
    Move::DasRight,
    Move::HardDrop,
    Move::None,
    Move::None,
  ],
  // Right
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Right,
    Move::SoftDrop,
    Move::DasLeft,
    Move::HardDrop,
    Move::None,
    Move::None,
  ],
  // Softdrop
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::DasLeft,
    Move::DasRight,
    Move::None,
    Move::None,
  ],
  // CCW
  [
    Move::CW, // clockwise after counter-clockwise allows for tsm
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::DasLeft,
    Move::DasRight,
    Move::HardDrop,
  ],
  // CW
  [
    Move::CW,
    Move::CCW, // counter-clockwise after clockwise allows for tsm
    Move::Flip,
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::DasLeft,
    Move::DasRight,
    Move::HardDrop,
  ],
  // Flip
  [
    Move::CW,
    Move::CCW,
    Move::Flip, // certain spins can only be done by doing a 180 twice
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::DasLeft,
    Move::DasRight,
    Move::HardDrop,
  ],
  // DasLeft
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Right,
    Move::SoftDrop,
    Move::DasRight,
    Move::HardDrop,
    Move::None,
    Move::None,
  ],
  // DasRight
  [
    Move::CW,
    Move::CCW,
    Move::Flip,
    Move::Left,
    Move::SoftDrop,
    Move::DasLeft,
    Move::HardDrop,
    Move::None,
    Move::None,
  ],
];

#[inline(always)]
pub fn compress_blocks(blocks: &[(u8, u8); 4]) -> [u64; 7] {
  let mut res = [0u64; 7];

  for &(x, y) in blocks {
    let idx = (y / 4) as usize;
    let bit = 1u64 << ((x * 4 + (y % 4)) as u64);
    res[idx] |= bit;
  }

  res
}

pub fn get_keys(mut state: Game, config: &GameConfig, target: (u8, u8, u8, Spin)) -> Vec<Move> {
  let mut passed = [0u64; 1024];

  let mut queue = [(0, 0, 0, Spin::None, ([Move::None; 16], 0usize)); 2048];

  let mut front_ptr = 0;
  let mut back_ptr = 1;

  let target_blocks = state
    .piece
    .mino
    .rot(target.2)
    .map(|block| (target.0 - block.0, target.1 - block.1));

  let target_compressed = compress_blocks(&target_blocks);

  let tgt_2 = target.2 % 2;

  queue[0] = (
    state.piece.x,
    state.piece.y,
    state.piece.rot,
    Spin::None,
    ([Move::None; 16], 0usize),
  );

  let game = state.clone();

  while front_ptr < back_ptr {
    let (x, y, rot, spin, moves) = queue[front_ptr];
    front_ptr += 1;

    for &mv in &MOVES[moves.0[moves.1.max(1) - 1] as usize] {
      if mv == Move::None {
        break;
      }

      state.spin = spin;
      state.piece.x = x;
      state.piece.y = y;
      state.piece.rot = rot;

      let fail = !mv.run(&mut state, config);

      if mv == Move::HardDrop {
        if state.piece.rot % 2 == tgt_2
          && state.spin == target.3
          && compress_blocks(
            &state
              .piece
              .blocks()
              .map(|block| (state.piece.x - block.0, state.piece.y - block.1)),
          ) == target_compressed
        {
          return Vec::from(&moves.0[0..moves.1])
            .into_iter()
            .chain(vec![mv])
            .collect();
        }
      } else {
        if fail || moves.1 >= 64 {
          continue;
        }

        let mut new_moves = moves.0;
        new_moves[moves.1] = mv;

        let compressed = 0u16
          | (state.piece.x as u16 & 0b_1111)
          | ((state.piece.y as u16 & 0b_111111) << 4)
          | ((state.piece.rot as u16 & 0b11) << 10)
          | ((state.spin as u16 & 0b11) << 12);

        let idx = compressed as usize / 64;
        let bit = 1 << (compressed % 64);

        if fail || passed[idx] & bit != 0 {
          continue;
        }

        passed[idx] |= bit;

        queue[back_ptr] = (
          state.piece.x,
          state.piece.y,
          state.piece.rot,
          state.spin,
          (new_moves, moves.1 + 1),
        );
        back_ptr += 1;
      }
    }
  }

  state.print();
  println!("Target:");
  state.piece.x = target.0;
  state.piece.y = target.1;
  state.piece.rot = target.2;
  state.spin = target.3;
  state.print();
  println!("Initial:");
  game.print();

  panic!("No move found (tgt spin: {})", target.3.as_str());
}
