use super::util::const_for_dynamic;
use crate::game2::{board::Board, config::ConstConfig, data::Mino, map::CollisionMap};

const fn kick_table_data<
  const CONFIG: ConstConfig,
  const PIECE: Mino,
  const ROT: usize,
  const AMT: u8,
  const I: usize,
>() -> (i8, i8) {
  CONFIG
    .kicktable
    .data(PIECE, ROT as u8, (ROT as u8 + AMT) & 3)[I]
}

const fn kick_table_data_x<
  const CONFIG: ConstConfig,
  const PIECE: Mino,
  const ROT: usize,
  const AMT: u8,
  const I: usize,
>() -> i8 {
  kick_table_data::<CONFIG, PIECE, ROT, AMT, I>().0
}
const fn kick_table_data_y<
  const CONFIG: ConstConfig,
  const PIECE: Mino,
  const ROT: usize,
  const AMT: u8,
  const I: usize,
>() -> i8 {
  kick_table_data::<CONFIG, PIECE, ROT, AMT, I>().1
}

const fn canonical_offset_x(piece: Mino, rot: usize) -> i8 {
  match piece {
    Mino::I => match rot {
      2 => 1,
      _ => 0,
    },
    Mino::S | Mino::Z => match rot {
      3 => 1,
      _ => 0,
    },
    _ => 0,
  }
}

const fn canonical_offset_y(piece: Mino, rot: usize) -> i8 {
  match piece {
    Mino::I => match rot {
      3 => -1,
      _ => 0,
    },
    Mino::S | Mino::Z => match rot {
      2 => 1,
      _ => 0,
    },
    _ => 0,
  }
}

struct PieceSearchSize<const P: Mino>;

impl<const P: Mino> PieceSearchSize<P> {
  const VALUE: usize = P.search_size() as usize;
}

struct KickTableSize<const C: ConstConfig>;

impl<const C: ConstConfig> KickTableSize<C> {
  const VALUE: usize = C.kicktable.raw().real_size;
}

#[derive(Debug, Clone)]
pub struct Bitset {
  bits: u8,
  mask: u8,
}

impl Bitset {
  pub const fn new(size: usize) -> Self {
    Bitset {
      bits: 0,
      mask: (1 << size) - 1,
    }
  }

  #[inline(always)]
  pub const fn any(&self) -> bool {
    (self.bits & self.mask) != 0
  }

  #[inline(always)]
  pub const fn all(&self) -> bool {
    (self.bits & self.mask) == self.mask
  }

  #[inline(always)]
  pub const fn none(&self) -> bool {
    (self.bits & self.mask) == 0
  }

  #[inline(always)]
  pub const fn not_all(&self) -> bool {
    (self.bits & self.mask) != self.mask
  }

  #[inline(always)]
  pub const fn on(&mut self, index: usize) {
    self.set(index, true);
  }

  #[inline(always)]
  pub const fn off(&mut self, index: usize) {
    self.set(index, false);
  }

  #[inline(always)]
  pub const fn set(&mut self, index: usize, value: bool) {
    if value {
      self.bits |= 1 << index;
    } else {
      self.bits &= !(1 << index);
    }
  }

  #[inline(always)]
  pub const fn set_all(&mut self, value: bool) {
    if value {
      self.bits = self.mask;
    } else {
      self.bits = 0;
    }
  }

  #[inline(always)]
  pub const fn get(&self, index: usize) -> bool {
    (self.bits & (1 << index)) != 0
  }
}

pub fn expand<const PIECE: Mino, const CONFIG: ConstConfig>(
  cmap: &CollisionMap,
  initial_pos: (usize, u8),
) -> [CollisionMap; 3] {
  let mut res = [CollisionMap::blank(); 3];

  let canidates = cmap.landable();

  let mut search = [Board::new(); 4];

  let mut remaining = Bitset::new(const { PIECE.real_permutations() });
  let mut complete = Bitset::new(const { PIECE.search_size() as usize });

  let spawn_y = 19u8;
  let h_spawn = PIECE.h_spawn();
  let board_y = initial_pos.1;

  let is_slow_init = Board::HEIGHT > spawn_y as u32 && board_y > spawn_y.saturating_sub(h_spawn);

  if is_slow_init {
    let threshold = std::cmp::min(spawn_y + 1, Board::HEIGHT as u8);
    let mut spawn = spawn_y;
    while spawn < threshold && !cmap[0].get(4, spawn) {
      spawn += 1;
    }

    if spawn == threshold {
      return res;
    }

    search[0].set(4, spawn);
    remaining.set_all(true);
    complete.set_all(true);
    complete.off(0);
  } else {
    for rot in 0..PIECE.real_permutations() {
      for x in 0..Board::WIDTH {
        let blocked = ((1u64 << Board::HEIGHT) - 1) & !cmap[rot][x];
        let fill = (1u64 << (u64::BITS - blocked.leading_zeros())) - 1;
        search[rot][x] = ((1u64 << Board::HEIGHT) - 1) ^ fill;
      }

      search[rot] |= (search[rot].shift_left() | search[rot].shift_right()) & cmap[rot];
      search[rot] |= (search[rot].shift_left() | search[rot].shift_right()) & cmap[rot];
    }

    let is_group3 = matches!(PIECE, Mino::T | Mino::L | Mino::J);
    if is_group3 {
      for rot in 0..4usize {
        let cw = (rot + 1) & 3;
        let ccw = (rot + 3) & 3;

        search[rot] |= (search[cw] | search[ccw]) & cmap[rot];
      }
    }

    for rot in 0..PIECE.real_permutations() {
      res[0][rot] |= search[rot] & canidates[rot];
      remaining.set(rot, res[0][rot] != canidates[rot]);
    }

    if remaining.none() {
      return res;
    }

    let is_group2 = matches!(PIECE, Mino::I | Mino::S | Mino::Z);
    if is_group2 {
      search[2] = search[0];
      search[3] = search[1];
    }
  }

  let mut unsearched: [Board; 4] =
    std::array::from_fn(|rot| !search[rot] & cmap[Mino::canonical_rot::<PIECE>(rot)]);

  // real loop here
  while complete.not_all() {
    const_for_dynamic!(R in 0..PieceSearchSize::<PIECE>::VALUE => {
      const ROT: usize = if R >= 4 {0} else {R};
      'rot_block: {
        if complete.get(ROT) {
          break 'rot_block;
        }

        complete.on(ROT);

        let rot_c = const { Mino::canonical_rot::<PIECE>(ROT) };

        loop {
          let tmp = (search[ROT].shift_left()
            | search[ROT].shift_right()
            | search[ROT].shift_down())
            & unsearched[ROT];

          if tmp.is_empty() {
            break;
          }

          search[ROT] |= tmp;
          unsearched[ROT] ^= tmp;
        }

        res[0][rot_c] |= search[ROT] & canidates[rot_c];
        remaining.set(rot_c, res[0][rot_c] != canidates[rot_c]);

        if remaining.none() {
          complete.set_all(true);
          break 'rot_block;
        }

        macro_rules! rotate {
          ($amt:expr) => {
            macro_rules! data_x {
              ($i:expr) => {
                kick_table_data_x::<CONFIG, PIECE, ROT, $amt, $i>()
              }
            }

            macro_rules! data_y {
              ($i:expr) => {
                kick_table_data_y::<CONFIG, PIECE, ROT, $amt, $i>()
              }
            }

            let rotated = const { (ROT + $amt as usize) & 3 };
            let rotated_c = Mino::canonical_rot::<PIECE>(rotated);

            let off_x = canonical_offset_x(PIECE, ROT) - canonical_offset_x(PIECE, rotated);
            let off_y = canonical_offset_y(PIECE, ROT) - canonical_offset_y(PIECE, rotated);

            let mut tmp = search[ROT];
            let mut result = Board::new();

            const_for_dynamic!(I_UNCHECKED in 0..KickTableSize::<CONFIG>::VALUE => {
              const I: usize = if I_UNCHECKED > 10 { 10 } else { I_UNCHECKED };
              let dx = kick_table_data_x::<CONFIG, PIECE, ROT, $amt, I>() + off_x;
              let dy = -kick_table_data_y::<CONFIG, PIECE, ROT, $amt, I>() + off_y;
              result |= tmp.shift(dx, dy);

              if I != CONFIG.kicktable.raw().real_size - 1 {
                tmp &= !(cmap[rotated_c].shift(-dx, -dy));
              }
            });

            result &= unsearched[rotated];

            if !result.is_empty() {
              search[rotated] |= result;
              unsearched[rotated] &= !result;

              complete.off(rotated);

              res[0][rotated_c] |= result & canidates[rotated_c];
              remaining.set(rotated_c, res[0][rotated_c] != canidates[rotated_c])
            }
          };
        }


        if PIECE != Mino::O {
          rotate!(1u8);
          if CONFIG.enable_180 {
            rotate!(2u8);
          }
          rotate!(3u8);

          if remaining.none() {
            complete.set_all(true);
            break 'rot_block;
          }
        }
      }
    });
  }

  res
}
