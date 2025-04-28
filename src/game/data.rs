use core::fmt;
use std::fmt::Formatter;

use serde::{Deserialize, Serialize};

use super::{Game, GameConfig};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mino {
  I,
  J,
  L,
  O,
  S,
  T,
  Z,
}

impl Mino {
  pub fn block_str(&self) -> &str {
    match self {
      Mino::I => "\x1b[46m  \x1b[49m",
      Mino::J => "\x1b[44m  \x1b[49m",
      Mino::L => "\x1b[43m  \x1b[49m",
      Mino::O => "\x1b[47m  \x1b[49m",
      Mino::S => "\x1b[102m  \x1b[49m",
      Mino::T => "\x1b[105m  \x1b[49m",
      Mino::Z => "\x1b[101m  \x1b[49m",
    }
  }
}

pub struct TetrominoMatrix {
  pub w: u8,
  pub rots: [[(u8, u8); 4]; 4],
}

impl Mino {
  pub fn data(&self) -> &TetrominoMatrix {
    match self {
      Mino::I => &TETROMINO_I,
      Mino::J => &TETROMINO_J,
      Mino::L => &TETROMINO_L,
      Mino::O => &TETROMINO_O,
      Mino::S => &TETROMINO_S,
      Mino::T => &TETROMINO_T,
      Mino::Z => &TETROMINO_Z,
    }
  }

  pub fn rot(&self, rot: u8) -> &[(u8, u8); 4] {
    debug_assert!(rot < 4, "Invalid rotation index: {}", rot);

    let rot = rot as usize;

    match self {
      Mino::I => &TETROMINO_I.rots[rot],
      Mino::J => &TETROMINO_J.rots[rot],
      Mino::L => &TETROMINO_L.rots[rot],
      Mino::O => &TETROMINO_O.rots[rot],
      Mino::S => &TETROMINO_S.rots[rot],
      Mino::T => &TETROMINO_T.rots[rot],
      Mino::Z => &TETROMINO_Z.rots[rot],
    }
  }

  pub fn str(&self) -> &str {
    match self {
      Mino::I => "I",
      Mino::J => "J",
      Mino::L => "L",
      Mino::O => "O",
      Mino::S => "S",
      Mino::T => "T",
      Mino::Z => "Z",
    }
  }
}

pub const TETROMINO_I: TetrominoMatrix = TetrominoMatrix {
  w: 4, // otherwise it doesn't spawn right
  rots: [
    [(0, 1), (1, 1), (2, 1), (3, 1)],
    [(1, 3), (1, 2), (1, 1), (1, 0)],
    [(3, 2), (2, 2), (1, 2), (0, 2)],
    [(2, 0), (2, 1), (2, 2), (2, 3)],
  ],
};

pub const TETROMINO_L: TetrominoMatrix = TetrominoMatrix {
  w: 3,
  rots: [
    [(0, 0), (0, 1), (1, 1), (2, 1)],
    [(0, 2), (1, 2), (1, 1), (1, 0)],
    [(2, 2), (2, 1), (1, 1), (0, 1)],
    [(2, 0), (1, 0), (1, 1), (1, 2)],
  ],
};

pub const TETROMINO_J: TetrominoMatrix = TetrominoMatrix {
  w: 3,
  rots: [
    [(2, 0), (0, 1), (1, 1), (2, 1)],
    [(0, 0), (1, 2), (1, 1), (1, 0)],
    [(0, 2), (2, 1), (1, 1), (0, 1)],
    [(2, 2), (1, 0), (1, 1), (1, 2)],
  ],
};

pub const TETROMINO_O: TetrominoMatrix = TetrominoMatrix {
  w: 2,
  rots: [
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 1), (0, 0), (1, 1), (1, 0)],
    [(1, 1), (0, 1), (1, 0), (0, 0)],
    [(1, 0), (1, 1), (0, 0), (0, 1)],
  ],
};

pub const TETROMINO_Z: TetrominoMatrix = TetrominoMatrix {
  w: 3,
  rots: [
    [(1, 0), (2, 0), (0, 1), (1, 1)],
    [(0, 1), (0, 0), (1, 2), (1, 1)],
    [(1, 2), (0, 2), (2, 1), (1, 1)],
    [(2, 1), (2, 2), (1, 0), (1, 1)],
  ],
};

pub const TETROMINO_T: TetrominoMatrix = TetrominoMatrix {
  w: 3,
  rots: [
    [(1, 0), (0, 1), (1, 1), (2, 1)],
    [(0, 1), (1, 2), (1, 1), (1, 0)],
    [(1, 2), (2, 1), (1, 1), (0, 1)],
    [(2, 1), (1, 0), (1, 1), (1, 2)],
  ],
};

pub const TETROMINO_S: TetrominoMatrix = TetrominoMatrix {
  w: 3,
  rots: [
    [(0, 0), (1, 0), (1, 1), (2, 1)],
    [(0, 2), (0, 1), (1, 1), (1, 0)],
    [(2, 2), (1, 2), (1, 1), (0, 1)],
    [(2, 0), (2, 1), (1, 1), (1, 2)],
  ],
};

pub enum KickTable {
  SRS,
  SRSPlus,
}

const INDEX_LOOKUP_TABLE: [[u8; 4]; 4] = [
  [255, 0, 8, 7],
  [1, 255, 2, 9],
  [10, 3, 255, 4],
  [6, 11, 5, 255],
];

impl KickTable {
  pub fn get_index(from: u8, to: u8) -> usize {
    INDEX_LOOKUP_TABLE[from as usize][to as usize] as usize
  }

  pub fn data(&self, mino: Mino, from: u8, to: u8) -> &[(i8, i8); 5] {
    let kick_table = match self {
      KickTable::SRS => &SRS_KICKS,
      KickTable::SRSPlus => &SRS_PLUS_KICKS,
    };
    match mino {
      Mino::I => &kick_table.i[KickTable::get_index(from, to)],
      _ => &kick_table.standard[KickTable::get_index(from, to)],
    }
  }
}

pub struct KickData {
  pub standard: [[(i8, i8); 5]; 12],
  pub i: [[(i8, i8); 5]; 12],
}

pub const SRS_KICKS: KickData = KickData {
  standard: [
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],    // 0->1
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],      // 1->0
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],      // 1->2
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],    // 2->1
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],       // 2->3
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],   // 3->2
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],   // 3->0
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],       // 0->3
    [(0, -1), (1, -1), (-1, -1), (1, 0), (-1, 0)],   // 0->2
    [(1, 0), (1, -2), (1, -1), (0, -2), (0, -1)],    // 1->3
    [(0, 1), (-1, 1), (1, 1), (-1, 0), (1, 0)],      // 2->0
    [(-1, 0), (-1, -2), (-1, -1), (0, -2), (0, -1)], // 3->1
  ],
  i: [
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)], // 0->1
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)], // 1->0
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)], // 1->2
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)], // 2->1
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)], // 2->3
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)], // 3->2
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)], // 3->0
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)], // 0->3
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],    // 0->2
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],    // 1->3
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],    // 2->0
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],    // 3->1
  ],
};

pub const SRS_PLUS_KICKS: KickData = KickData {
  standard: [
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],    // 0->1
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],      // 1->0
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],      // 1->2
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],    // 2->1
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],       // 2->3
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],   // 3->2
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],   // 3->0
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],       // 0->3
    [(0, -1), (1, -1), (-1, -1), (1, 0), (-1, 0)],   // 0->2
    [(1, 0), (1, -2), (1, -1), (0, -2), (0, -1)],    // 1->3
    [(0, 1), (-1, 1), (1, 1), (-1, 0), (1, 0)],      // 2->0
    [(-1, 0), (-1, -2), (-1, -1), (0, -2), (0, -1)], // 3->1
  ],
  i: [
    [(0, 0), (1, 0), (-2, 0), (-2, 1), (1, -2)], // 0->1
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)], // 1->0
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)], // 1->2
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)], // 2->1
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)], // 2->3
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)], // 3->2
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)], // 3->0
    [(0, 0), (-1, 0), (2, 0), (2, 1), (-1, -2)], // 0->3
    [(0, 0), (0, -1), (0, 0), (0, 0), (0, 0)],   // 0->2
    [(0, 0), (1, 0), (0, 0), (0, 0), (0, 0)],    // 1->3
    [(0, 0), (0, 1), (0, 0), (0, 0), (0, 0)],    // 2->0
    [(0, 0), (-1, 0), (0, 0), (0, 0), (0, 0)],   // 3->1
  ],
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spin {
  None,
  Mini,
  Normal,
}

impl Spin {
  pub fn str(&self) -> &str {
    match self {
      Spin::None => "none",
      Spin::Mini => "mini",
      Spin::Normal => "normal",
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spins {
  None,
  T,
  TPlus,
  Mini,
  MiniPlus,
  All,
  AllPlus,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComboTable {
  None,
  Classic,
  Modern,
  Multiplier,
}

impl ComboTable {
  pub fn get(&self) -> &[u8] {
    assert_ne!(
      *self,
      ComboTable::Multiplier,
      "Multiplier combo table is not defined"
    );
    match self {
      ComboTable::None => &[0],
      ComboTable::Classic => &[0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5],
      ComboTable::Modern => &[0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4],
      ComboTable::Multiplier => panic!("Multiplier combo table is not defined"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[repr(u8)]
pub enum Move {
  None,
  Left,
  Right,
  SoftDrop,
  CCW,
  CW,
  Flip,
  DasLeft,
  DasRight,
	Hold,
	HardDrop,
}

impl Move {
  #[inline(always)]
  pub fn run(&self, game: &mut Game, config: &GameConfig) -> bool {
    match self {
      Move::Left => game.move_left(),
      Move::Right => game.move_right(),
      Move::SoftDrop => game.soft_drop(),
      Move::CCW => game.rotate(3, config).0,
      Move::CW => game.rotate(1, config).0,
      Move::Flip => game.rotate(2, config).0,
      Move::None => panic!("None move called...cf"),
      Move::DasLeft => game.das_left(),
      Move::DasRight => game.das_right(),
			Move::Hold => game.hold(),
			Move::HardDrop => {
				game.soft_drop();
				true
			}
    }
  }

  pub fn str(&self) -> &str {
    match self {
      Move::None => "none",
      Move::Left => "left",
      Move::Right => "right",
      Move::SoftDrop => "soft drop",
      Move::CCW => "ccw",
      Move::CW => "cw",
      Move::Flip => "180",
      Move::DasLeft => "das left",
      Move::DasRight => "das right",
			Move::Hold => "hold",
			Move::HardDrop => "hard drop",
    }
  }

	pub fn triangle_key(&self) -> &str {
    match self {
      Move::None => panic!("This move doesn't exist"),
      Move::Left => "moveLeft",
      Move::Right => "moveRight",
      Move::SoftDrop => "softDrop",
      Move::CCW => "rotateCCW",
      Move::CW => "rotateCW",
      Move::Flip => "rotate180",
      Move::DasLeft => "dasLeft",
      Move::DasRight => "dasRight",
			Move::Hold => "hold",
			Move::HardDrop => "hardDrop",
    }
  }
}
