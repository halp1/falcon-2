use crate::game::{
  Game, GameConfig,
  data::{Move, Spin},
};

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
    Move::Left,
    Move::Right,
    Move::SoftDrop,
    Move::DasLeft,
    Move::DasRight,
    Move::HardDrop,
    Move::None,
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

      if mv == Move::HardDrop {
        if state.piece.rot % 2 == tgt_2
          && state.spin == target.3
          && state
            .piece
            .blocks()
            .map(|block| (state.piece.x - block.0, state.piece.y - block.1))
            .iter()
            .all(|block| {
              target_blocks
                .iter()
                .any(|tgt| block.0 == tgt.0 && block.1 == tgt.1)
            })
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

  panic!("No move found (tgt spin: {})", target.3.str());
}
