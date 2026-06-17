use const_for::const_for;
use serde::{Deserialize, Serialize};
use std::marker::ConstParamTy;

// use super::{Game, GameConfig};

#[derive(Debug)]
pub struct TetrominoMatrix {
  pub w: u8,
  pub rots: [[(i8, i8); 4]; 4],
}

#[derive(ConstParamTy, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
  pub const fn real_permutations(self) -> usize {
    match self {
      Mino::I | Mino::S | Mino::Z => 2,
      Mino::O => 1,
      _ => 4,
    }
  }

  #[inline(always)]
  pub const fn canonical_rot<const PIECE: Mino>(rot: usize) -> usize {
    match PIECE.real_permutations() {
      1 => 0,
      2 => rot & 1,
      _ => rot,
    }
  }

  #[inline(always)]
  pub const fn block_str(&self) -> &str {
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

  #[inline(always)]
  pub const fn data(&self) -> &TetrominoMatrix {
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
  pub const fn rot(&self, rot: u8) -> &[(i8, i8); 4] {
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
  pub const fn corner_table(&self, rot: u8) -> Option<&CornerTable> {
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

  pub const fn search_size(self) -> u8 {
    match self {
      Mino::O => 1,
      _ => 4,
    }
  }

  pub const fn size_of<const PIECE: Mino>() -> usize {
    PIECE.data().w as usize
  }

  pub const fn str(&self) -> &str {
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

  #[inline(always)]
  pub const fn h_gen(self) -> u8 {
    match self {
      Mino::I | Mino::T => 2,
      Mino::O => 0,
      _ => 1,
    }
  }

  #[inline(always)]
  pub const fn h_spawn(self) -> u8 {
    match self {
      Mino::I => 2,
      Mino::O => 0,
      _ => 1,
    }
  }

  pub fn from_char(c: char) -> Self {
    match c.to_ascii_uppercase() {
      'I' => Mino::I,
      'T' => Mino::T,
      'O' => Mino::O,
      'J' => Mino::J,
      'L' => Mino::L,
      'S' => Mino::S,
      'Z' => Mino::Z,
      _ => panic!("Invalid mino char: {}", c),
    }
  }
}

const fn make_matrix(size: u8, initial: [(i8, i8); 4]) -> TetrominoMatrix {
  let mut rots = [initial; 4];

  const_for!(i in 1..4 => {
    const_for!(j in 0..4 => {
      // Cleanly grab the block from the PREVIOUS rotation state
      let prev_block = rots[i - 1][j];

      // Apply the 90-degree rotation mapping
      rots[i][j] = (prev_block.1, -prev_block.0);
    });
  });

  TetrominoMatrix { w: size, rots }
}

pub const TETROMINO_L: TetrominoMatrix = make_matrix(3, [(-1, 0), (0, 0), (1, 0), (1, 1)]);

pub const TETROMINO_J: TetrominoMatrix = make_matrix(3, [(-1, 0), (0, 0), (1, 0), (-1, 1)]);

pub const TETROMINO_Z: TetrominoMatrix = make_matrix(3, [(-1, 1), (0, 1), (0, 0), (1, 0)]);

pub const TETROMINO_T: TetrominoMatrix = make_matrix(3, [(-1, 0), (0, 0), (0, 1), (1, 0)]);

pub const TETROMINO_S: TetrominoMatrix = make_matrix(3, [(-1, 0), (0, 0), (0, 1), (1, 1)]);

pub const TETROMINO_O: TetrominoMatrix = TetrominoMatrix {
  w: 2,
  rots: [[(0, 0), (0, 1), (1, 0), (1, 1)]; 4],
};

pub const TETROMINO_I: TetrominoMatrix = make_matrix(5, [(-1, 0), (0, 0), (1, 0), (2, 0)]);

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

const INDEX_LOOKUP_TABLE: [[u8; 4]; 4] = [
  [255, 0, 8, 7],
  [1, 255, 2, 9],
  [10, 3, 255, 4],
  [6, 11, 5, 255],
];

#[derive(ConstParamTy, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KickTable {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "SRS")]
  SRS,
  #[serde(rename = "SRS+")]
  SRSPlus,
  #[serde(rename = "SRS-X")]
  SRSX,
}

impl KickTable {
  #[inline(always)]
  pub const fn get_index(from: u8, to: u8) -> usize {
    INDEX_LOOKUP_TABLE[from as usize][to as usize] as usize
  }

  #[inline(always)]
  pub const fn raw(&self) -> &KickData {
    match self {
      KickTable::None => &NONE_KICKS,
      KickTable::SRS => &SRS_KICKS,
      KickTable::SRSPlus => &SRS_PLUS_KICKS,
      KickTable::SRSX => &SRS_X_KICKS,
    }
  }

  #[inline(always)]
  pub const fn data(&self, mino: Mino, from: u8, to: u8) -> &[(i8, i8); KICKTABLE_SIZE] {
    match mino {
      Mino::I => &self.raw().i[KickTable::get_index(from, to)],
      _ => &self.raw().standard[KickTable::get_index(from, to)],
    }
  }
}

const KICKTABLE_SIZE: usize = 11;

pub struct KickData {
  pub real_size: usize,
  pub standard: [[(i8, i8); KICKTABLE_SIZE]; 12],
  pub i: [[(i8, i8); KICKTABLE_SIZE]; 12],
}

const I_OFFSET_TABLE: [(i8, i8); 4] = [(0, 0), (-1, 0), (-1, -1), (0, -1)];

impl KickData {
  /// modify i kicks to work with true rotation
  pub const fn convert_i_kicks(&self) -> Self {
    let mut i = self.i;
    const_for!(from in 0..4i8 => {
      const_for!(to in 0..4i8 => {
        if from != to {
          let index = KickTable::get_index(from as u8, to as u8);

          const_for!(kick_idx in 0..KICKTABLE_SIZE => {
            {
              let o1 = I_OFFSET_TABLE[from as usize];
              let o2 = I_OFFSET_TABLE[to as usize];
              i[index][kick_idx].0 += o1.0 - o2.0;
              i[index][kick_idx].1 += o1.1 - o2.1;
            }
          });

        }
      });
    });

    Self {
      real_size: self.real_size,
      standard: self.standard,
      i,
    }
  }
}

pub const NONE_KICKS: KickData = KickData {
  real_size: 1,
  standard: [[(0, 0); KICKTABLE_SIZE]; 12],
  i: [[(0, 0); KICKTABLE_SIZE]; 12],
};

pub const SRS_KICKS: KickData = KickData {
  real_size: 5,
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
}
.convert_i_kicks();

pub const SRS_PLUS_KICKS: KickData = KickData {
  real_size: 5,
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
}
.convert_i_kicks();

pub const SRS_X_KICKS: KickData = KickData {
  real_size: KICKTABLE_SIZE,
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
}
.convert_i_kicks();

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
  // pub fn run(
  //   &self,
  //   game: &mut Game,
  //   config: &GameConfig,
  //   map: &CollisionMap,
  //   start: &StartState,
  // ) -> bool {
  //   match self {
  //     Move::Left => game.move_left(&map),
  //     Move::Right => game.move_right(&map),
  //     Move::SoftDrop => game.soft_drop(&map),
  //     Move::CCW => game.rotate(3, config, &map).0,
  //     Move::CW => game.rotate(1, config, &map).0,
  //     Move::Flip => game.rotate(2, config, &map).0,
  //     Move::None => panic!("None move called...cf"),
  //     Move::DasLeft => game.das_left(&map),
  //     Move::DasRight => game.das_right(&map),
  //     Move::Hold => game.hold(start),
  //     Move::HardDrop => {
  //       game.soft_drop(&map);
  //       true
  //     }
  //   }
  // }

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
