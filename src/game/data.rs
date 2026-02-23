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
  #[inline(always)]
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

  #[inline(always)]
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

	#[inline(always)]
	pub fn corner_table(&self, rot: u8) -> Option<&CornerTable> {
    debug_assert!(rot < 4, "Invalid rotation index: {}", rot);

		match self {
			Mino::I => None,
			Mino::J => Some(&CORNERTABLE_J),
			Mino::L => Some(&CORNERTABLE_L),
			Mino::O => None,
			Mino::S => Some(&CORNERTABLE_S),
			Mino::T => Some(&CORNERTABLE_T),
			Mino::Z => Some(&CORNERTABLE_Z),
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



pub type CornerTable = [[((i8, i8), Option<(u8, u8)>); 4]; 4];

pub const CORNERTABLE_Z: CornerTable = [
  [
    ((4, 1), None),
    ((1, 1), None),
    ((0, 0), None),
    ((3, 0), None),
  ],
  [
    ((2, 1), None),
    ((1, 2), None),
    ((2, -2), None),
    ((1, -1), None),
  ],
  [
    ((4, 0), None),
    ((1, 0), None),
    ((0, -1), None),
    ((3, -1), None),
  ],
  [
    ((3, 1), None),
    ((2, 2), None),
    ((2, -1), None),
    ((3, -2), None),
  ],
];

pub const CORNERTABLE_L: CornerTable = [
  [
    ((3, 1), None),
    ((2, 1), None),
    ((1, -1), None),
    ((3, -1), None),
  ],
  [
    ((3, 1), None),
    ((1, 1), None),
    ((1, 0), None),
    ((3, -1), None),
  ],
  [
    ((3, 1), None),
    ((1, 1), None),
    ((1, -1), None),
    ((2, -1), None),
  ],
  [
    ((3, 0), None),
    ((1, 1), None),
    ((1, -1), None),
    ((3, -1), None),
  ],
];

pub const CORNERTABLE_S: CornerTable = [
  [
    ((3, 1), None),
    ((0, 1), None),
    ((1, 0), None),
    ((4, 0), None),
  ],
  [
    ((2, 2), None),
    ((1, 1), None),
    ((1, -2), None),
    ((2, -1), None),
  ],
  [
    ((3, 0), None),
    ((0, 0), None),
    ((1, -1), None),
    ((4, -1), None),
  ],
  [
    ((3, 2), None),
    ((2, 1), None),
    ((3, -1), None),
    ((2, -2), None),
  ],
];

pub const CORNERTABLE_J: CornerTable = [
  [
    ((2, 1), None),
    ((1, 1), None),
    ((1, -1), None),
    ((3, -1), None),
  ],
  [
    ((3, 1), None),
    ((1, 0), None),
    ((1, -1), None),
    ((3, -1), None),
  ],
  [
    ((3, 1), None),
    ((1, 1), None),
    ((2, -1), None),
    ((3, -1), None),
  ],
  [
    ((3, 1), None),
    ((1, 1), None),
    ((1, -1), None),
    ((3, 0), None),
  ],
];

pub const CORNERTABLE_T: CornerTable = [
  [
    ((3, 1), Some((3, 0))),
    ((1, 1), Some((0, 1))),
    ((1, -1), Some((1, 2))),
    ((3, -1), Some((2, 3))),
  ],
  [
    ((3, 1), Some((3, 0))),
    ((1, 1), Some((0, 1))),
    ((1, -1), Some((1, 2))),
    ((3, -1), Some((2, 3))),
  ],
  [
    ((3, 1), Some((3, 0))),
    ((1, 1), Some((0, 1))),
    ((1, -1), Some((1, 2))),
    ((3, -1), Some((2, 3))),
  ],
  [
    ((3, 1), Some((3, 0))),
    ((1, 1), Some((0, 1))),
    ((1, -1), Some((1, 2))),
    ((3, -1), Some((2, 3))),
  ],
];

#[derive(Deserialize, Clone)]
pub enum KickTable {
  #[serde(rename = "SRS")]
  SRS,
  #[serde(rename = "SRS+")]
  SRSPlus,
  #[serde(rename = "SRS-X")]
  SRSX,
}

const INDEX_LOOKUP_TABLE: [[u8; 4]; 4] = [
  [255, 0, 8, 7],
  [1, 255, 2, 9],
  [10, 3, 255, 4],
  [6, 11, 5, 255],
];

impl KickTable {
  #[inline(always)]
  pub fn get_index(from: u8, to: u8) -> usize {
    INDEX_LOOKUP_TABLE[from as usize][to as usize] as usize
  }

  #[inline(always)]
  pub fn data(&self, mino: Mino, from: u8, to: u8) -> &[(i8, i8); 11] {
    let kick_table = match self {
      KickTable::SRS => &SRS_KICKS,
      KickTable::SRSPlus => &SRS_PLUS_KICKS,
      KickTable::SRSX => &SRS_X_KICKS,
    };
    match mino {
      Mino::I => &kick_table.i[KickTable::get_index(from, to)],
      _ => &kick_table.standard[KickTable::get_index(from, to)],
    }
  }
}

pub struct KickData {
  pub standard: [[(i8, i8); 11]; 12],
  pub i: [[(i8, i8); 11]; 12],
}

pub const SRS_KICKS: KickData = KickData {
  standard: [
    [
      (0, 0),
      (-1, 0),
      (-1, -1),
      (0, 2),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->1
    [
      (0, 0),
      (1, 0),
      (1, 1),
      (0, -2),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->0
    [
      (0, 0),
      (1, 0),
      (1, 1),
      (0, -2),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->2
    [
      (0, 0),
      (-1, 0),
      (-1, -1),
      (0, 2),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->1
    [
      (0, 0),
      (1, 0),
      (1, -1),
      (0, 2),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->3
    [
      (0, 0),
      (-1, 0),
      (-1, 1),
      (0, -2),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->2
    [
      (0, 0),
      (-1, 0),
      (-1, 1),
      (0, -2),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->0
    [
      (0, 0),
      (1, 0),
      (1, -1),
      (0, 2),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->3
    [
      (0, -1),
      (1, -1),
      (-1, -1),
      (1, 0),
      (-1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->2
    [
      (1, 0),
      (1, -2),
      (1, -1),
      (0, -2),
      (0, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->3
    [
      (0, 1),
      (-1, 1),
      (1, 1),
      (-1, 0),
      (1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->0
    [
      (-1, 0),
      (-1, -2),
      (-1, -1),
      (0, -2),
      (0, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->1
  ],
  i: [
    [
      (0, 0),
      (-2, 0),
      (1, 0),
      (-2, 1),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->1
    [
      (0, 0),
      (2, 0),
      (-1, 0),
      (2, -1),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->0
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (-1, -2),
      (2, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->2
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (1, 2),
      (-2, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->1
    [
      (0, 0),
      (2, 0),
      (-1, 0),
      (2, -1),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->3
    [
      (0, 0),
      (-2, 0),
      (1, 0),
      (-2, 1),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->2
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (1, 2),
      (-2, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->0
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (-1, -2),
      (2, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->3
    [
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->2
    [
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->3
    [
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->0
    [
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->1
  ],
};

pub const SRS_PLUS_KICKS: KickData = KickData {
  standard: [
    [
      (0, 0),
      (-1, 0),
      (-1, -1),
      (0, 2),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->1
    [
      (0, 0),
      (1, 0),
      (1, 1),
      (0, -2),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->0
    [
      (0, 0),
      (1, 0),
      (1, 1),
      (0, -2),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->2
    [
      (0, 0),
      (-1, 0),
      (-1, -1),
      (0, 2),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->1
    [
      (0, 0),
      (1, 0),
      (1, -1),
      (0, 2),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->3
    [
      (0, 0),
      (-1, 0),
      (-1, 1),
      (0, -2),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->2
    [
      (0, 0),
      (-1, 0),
      (-1, 1),
      (0, -2),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->0
    [
      (0, 0),
      (1, 0),
      (1, -1),
      (0, 2),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->3
    [
      (0, -1),
      (1, -1),
      (-1, -1),
      (1, 0),
      (-1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->2
    [
      (1, 0),
      (1, -2),
      (1, -1),
      (0, -2),
      (0, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->3
    [
      (0, 1),
      (-1, 1),
      (1, 1),
      (-1, 0),
      (1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->0
    [
      (-1, 0),
      (-1, -2),
      (-1, -1),
      (0, -2),
      (0, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->1
  ],
  i: [
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (-2, 1),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->1
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (-1, 2),
      (2, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->0
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (-1, -2),
      (2, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->2
    [
      (0, 0),
      (-2, 0),
      (1, 0),
      (-2, -1),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->1
    [
      (0, 0),
      (2, 0),
      (-1, 0),
      (2, -1),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->3
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (1, -2),
      (-2, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->2
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (1, 2),
      (-2, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->0
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (2, 1),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->3
    [
      (0, 0),
      (0, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->2
    [
      (0, 0),
      (1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->3
    [
      (0, 0),
      (0, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->0
    [
      (0, 0),
      (-1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->1
  ],
};

pub const SRS_X_KICKS: KickData = KickData {
  standard: [
    [
      (0, 0),
      (-1, 0),
      (-1, -1),
      (0, 2),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->1
    [
      (0, 0),
      (1, 0),
      (1, 1),
      (0, -2),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->0
    [
      (0, 0),
      (1, 0),
      (1, 1),
      (0, -2),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->2
    [
      (0, 0),
      (-1, 0),
      (-1, -1),
      (0, 2),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->1
    [
      (0, 0),
      (1, 0),
      (1, -1),
      (0, 2),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->3
    [
      (0, 0),
      (-1, 0),
      (-1, 1),
      (0, -2),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->2
    [
      (0, 0),
      (-1, 0),
      (-1, 1),
      (0, -2),
      (-1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->0
    [
      (0, 0),
      (1, 0),
      (1, -1),
      (0, 2),
      (1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->3
    [
      (0, 0),
      (1, 0),
      (2, 0),
      (1, 1),
      (2, 1),
      (-1, 0),
      (-2, 0),
      (-1, 1),
      (-2, 1),
      (0, -1),
      (3, 0),
    ], // 0->2
    [
      (0, 0),
      (0, 1),
      (0, 2),
      (-1, 1),
      (-1, 2),
      (0, -1),
      (0, -2),
      (-1, -1),
      (-1, -2),
      (1, 0),
      (0, 3),
    ], // 1->3
    [
      (0, 0),
      (-1, 0),
      (-2, 0),
      (-1, -1),
      (-2, -1),
      (1, 0),
      (2, 0),
      (1, -1),
      (2, -1),
      (0, 1),
      (-3, 0),
    ], // 2->0
    [
      (0, 0),
      (0, 1),
      (0, 2),
      (1, 1),
      (1, 2),
      (0, -1),
      (0, -2),
      (1, -1),
      (1, -2),
      (-1, 0),
      (0, 3),
    ], // 3->1
  ],
  i: [
    [
      (0, 0),
      (-2, 0),
      (1, 0),
      (-2, 1),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->1
    [
      (0, 0),
      (2, 0),
      (-1, 0),
      (2, -1),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->0
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (-1, -2),
      (2, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->2
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (1, 2),
      (-2, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->1
    [
      (0, 0),
      (2, 0),
      (-1, 0),
      (2, -1),
      (-1, 2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->3
    [
      (0, 0),
      (-2, 0),
      (1, 0),
      (-2, 1),
      (1, -2),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->2
    [
      (0, 0),
      (1, 0),
      (-2, 0),
      (1, 2),
      (-2, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->0
    [
      (0, 0),
      (-1, 0),
      (2, 0),
      (-1, -2),
      (2, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->3
    [
      (0, 0),
      (-1, 0),
      (-2, 0),
      (1, 0),
      (2, 0),
      (0, 1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 0->2
    [
      (0, 0),
      (0, 1),
      (0, 2),
      (0, -1),
      (0, -2),
      (-1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 1->3
    [
      (0, 0),
      (1, 0),
      (2, 0),
      (-1, 0),
      (-2, 0),
      (0, -1),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 2->0
    [
      (0, 0),
      (0, 1),
      (0, 2),
      (0, -1),
      (0, -2),
      (1, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
      (0, 0),
    ], // 3->1
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

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Spins {
  None,
  #[serde(rename = "T-spins")]
  T,
  #[serde(rename = "T-spins+")]
  TPlus,
  #[serde(rename = "all-mini")]
  Mini,
  #[serde(rename = "all-mini+")]
  MiniPlus,
  #[serde(rename = "all")]
  All,
  #[serde(rename = "all+")]
  AllPlus,
  #[serde(rename = "mini-only")]
  MiniOnly,
  #[serde(rename = "handheld")]
  Handheld,
  #[serde(rename = "stupid")]
  Stupid,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ComboTable {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "classic-guideline")]
  Classic,
  #[serde(rename = "modern-guideline")]
  Modern,
  #[serde(rename = "multiplier")]
  Multiplier,
}

impl ComboTable {
  pub fn get(&self) -> &[u8] {
    debug_assert_ne!(
      *self,
      ComboTable::Multiplier,
      "Multiplier combo table is not defined"
    );
    match self {
      ComboTable::None => &[0],
      ComboTable::Classic => &[0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5],
      ComboTable::Modern => &[0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4],
      ComboTable::Multiplier => unreachable!("Multiplier combo table is not defined"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[repr(u8)]
pub enum Move {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "moveLeft")]
  Left,
  #[serde(rename = "moveRight")]
  Right,
  #[serde(rename = "softDrop")]
  SoftDrop,
  #[serde(rename = "rotateCCW")]
  CCW,
  #[serde(rename = "rotateCW")]
  CW,
  #[serde(rename = "rotate180")]
  Flip,
  #[serde(rename = "dasLeft")]
  DasLeft,
  #[serde(rename = "dasRight")]
  DasRight,
  #[serde(rename = "hold")]
  Hold,
  #[serde(rename = "hardDrop")]
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
