#[derive(Debug, Copy, Clone)]
pub struct Board {
  pub data: [u64; Self::WIDTH],
}

impl Board {
  pub const WIDTH: usize = 10;
  pub const HEIGHT: u32 = 40;
  pub const BUFFER: u32 = 20;

  pub const fn new() -> Self {
    Self {
      data: [0; Self::WIDTH],
    }
  }

  pub fn set(&mut self, x: usize, y: u8) {
    assert!(
      x < Self::WIDTH,
      "x value of {} is out of bounds [0, {})",
      x,
      Self::WIDTH
    );
    assert!(
      (y as u32) < Self::HEIGHT,
      "y value of {} is out of bounds [0, {})",
      y,
      Self::HEIGHT
    );
    self.data[x] |= 1u64 << y;
  }

  #[inline(always)]
  pub fn get(&self, x: usize, y: u8) -> bool {
    (self.data[x] & (1u64 << y)) != 0
  }

  #[inline(always)]
  pub fn shift(&self, dx: i8, dy: i8) -> Self {
    let mut board = Self::new();

    for i in 0..(Self::WIDTH - dx.abs() as usize) {
      board[i + if dx >= 0 { dx } else { 0 } as usize] = {
        let v = self[i + if dx < 0 { -dx } else { 0 } as usize];

        if dy > 0 {
          v << dy
        } else if dy < 0 {
          v >> -dy
        } else {
          v
        }
      };
    }

    board
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    !self.data.iter().any(|&f| f != 0)
  }

  pub fn real_height(&self) -> u32 {
    u64::BITS
      - self
        .data
        .iter()
        .reduce(|a, b| a.max(b))
        .unwrap()
        .leading_zeros()
  }

  pub fn str_vec(&self, height: u32) -> Vec<String> {
    (0..height)
      .rev()
      .map(|h| {
        let mut str = String::new();
        str += &format!("{:2}|", h + 1);
        for (_x, col) in self.data.iter().enumerate() {
          if (col & (1u64 << h)) != 0 {
            str += &format!("\x1B[100m  \x1B[49m");
          } else {
            str += &format!("  ");
          }
        }
        str += &format!("|");
        str.to_string()
      })
      .collect()
  }

  pub fn print(&self) {
    self
      .str_vec(self.real_height().max(4))
      .iter()
      .for_each(|str| println!("{}", str));
  }

  pub fn count_ones(&self) -> u32 {
    self.data.iter().map(|c| c.count_ones()).sum()
  }

  #[inline(always)]
  fn clear_setup(&mut self, garbage_level: u8) -> (u64, u8) {
    let clear_mask = self
      .data
      .into_iter()
      .reduce(|acc, col| acc & col)
      .unwrap_or(0);

    if clear_mask == 0 {
      return (0, 0);
    }

    let garbage_cleared = if garbage_level > 0 {
      (clear_mask & ((1u64 << garbage_level) - 1)).count_ones() as u8
    } else {
      0
    };

    (clear_mask, garbage_cleared)
  }

  #[cfg(target_feature = "bmi2")]
  pub fn clear(&mut self, garbage_level: u8) -> (u8, u8) {
    use std::arch::x86_64::*;

    let (clear_mask, garbage_cleared) = self.clear_setup(garbage_level);

    if clear_mask == 0 {
      return (0, 0);
    }

    self
      .data
      .iter_mut()
      .for_each(|col| unsafe { *col = _pext_u64(*col, !clear_mask) });

    return (clear_mask.count_ones() as u8, garbage_cleared);
  }

  #[cfg(not(target_feature = "bmi2"))]
  pub fn clear(&mut self, garbage_level: u8) -> (u8, u8) {
    let (clear_mask, garbage_cleared) = self.clear_setup(garbage_level);

    if clear_mask == 0 {
      return (0, 0);
    }

    while clear_mask != 0 {
      let y = 63 - clear_mask.leading_zeros();
      for x in FULL_WIDTH {
        let low_mask = (1u64 << y) - 1;
        let low = self.data[x] & low_mask;
        let high = self.data[x] >> (y + 1);
        self.data[x] = (high << y) | low;
      }

      clear_mask ^= 1u64 << y;
    }

    (clear_mask.count_ones() as u8, garbage_cleared)
  }
}

use std::ops::{
  BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
  ShrAssign,
};

impl BitOr for Board {
  type Output = Board;
  fn bitor(self, rhs: Self) -> Self::Output {
    Board {
      data: std::array::from_fn(|i| self.data[i] | rhs.data[i]),
    }
  }
}

impl BitAnd for Board {
  type Output = Board;
  fn bitand(self, rhs: Self) -> Self::Output {
    Board {
      data: std::array::from_fn(|i| self.data[i] & rhs.data[i]),
    }
  }
}

impl BitXor for Board {
  type Output = Board;
  fn bitxor(self, rhs: Self) -> Self::Output {
    Board {
      data: std::array::from_fn(|i| self.data[i] ^ rhs.data[i]),
    }
  }
}

impl Not for Board {
  type Output = Board;
  fn not(self) -> Self::Output {
    Board {
      data: std::array::from_fn(|i| !self.data[i]),
    }
  }
}

impl BitOrAssign for Board {
  fn bitor_assign(&mut self, rhs: Self) {
    for i in 0..self.data.len() {
      self.data[i] |= rhs.data[i];
    }
  }
}

impl BitAndAssign for Board {
  fn bitand_assign(&mut self, rhs: Self) {
    for i in 0..self.data.len() {
      self.data[i] &= rhs.data[i];
    }
  }
}

impl BitXorAssign for Board {
  fn bitxor_assign(&mut self, rhs: Self) {
    for i in 0..self.data.len() {
      self.data[i] ^= rhs.data[i];
    }
  }
}

impl Shl<usize> for Board {
  type Output = Board;
  fn shl(self, rhs: usize) -> Self::Output {
    Board {
      data: std::array::from_fn(|i| self.data[i] << rhs),
    }
  }
}

impl Shr<usize> for Board {
  type Output = Board;
  fn shr(self, rhs: usize) -> Self::Output {
    Board {
      data: std::array::from_fn(|i| self.data[i] >> rhs),
    }
  }
}

impl ShlAssign<usize> for Board {
  fn shl_assign(&mut self, rhs: usize) {
    for i in 0..self.data.len() {
      self.data[i] <<= rhs;
    }
  }
}

impl ShrAssign<usize> for Board {
  fn shr_assign(&mut self, rhs: usize) {
    for i in 0..self.data.len() {
      self.data[i] >>= rhs;
    }
  }
}

impl PartialEq for Board {
  fn eq(&self, other: &Self) -> bool {
    self.data == other.data
  }
}

impl Eq for Board {}

use std::ops::{Index, IndexMut};

#[cfg(not(target_feature = "bmi2"))]
use triangle::engine::events::garbage;

const impl Index<usize> for Board {
  type Output = u64;

  #[inline]
  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}

const impl IndexMut<usize> for Board {
  #[inline]
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.data[index]
  }
}
