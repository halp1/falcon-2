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
  pub fn set_unchecked(&mut self, x: usize, y: u8) {
    unsafe {
      *self.data.get_unchecked_mut(x) |= 1u64 << y;
    }
  }

  #[inline(always)]
  pub fn clear_lines(&mut self, mut lines: u64) {
    loop {
      let mask = !(((lines as i64 & -(lines as i64)) - 1) as u64);
      for x in 0..Self::WIDTH {
        unsafe {
          let col = *self.data.get_unchecked(x);
          *self.data.get_unchecked_mut(x) = col ^ ((col ^ (col >> 1)) & mask);
        }
      }
      lines = (lines & (lines - 1)) >> 1;
      if lines == 0 {
        break;
      }
    }
  }

  #[inline(always)]
  pub const fn shift_const<const DX: i8, const DY: i8>(&self) -> Self {
    let mut board = Self::new();
    if DX >= Self::WIDTH as i8
      || DX <= -(Self::WIDTH as i8)
      || DY >= Self::HEIGHT as i8
      || DY <= -(Self::HEIGHT as i8)
    {
      return board;
    }

    if DX == 0 && DY == 0 {
      return *self;
    }

    if DX >= 0 {
      let adx = DX as usize;
      let limit = Self::WIDTH - adx;
      let mut i = 0;
      while i < limit {
        let val = self.data[i];
        board.data[i + adx] = if DY > 0 {
          val << DY
        } else if DY < 0 {
          val >> -DY
        } else {
          val
        };
        i += 1;
      }
    } else {
      let adx = -DX as usize;
      let limit = Self::WIDTH - adx;
      let mut i = 0;
      while i < limit {
        let val = self.data[i + adx];
        board.data[i] = if DY > 0 {
          val << DY
        } else if DY < 0 {
          val >> -DY
        } else {
          val
        };
        i += 1;
      }
    }

    board
  }

  #[inline(always)]
  pub fn get(&self, x: usize, y: u8) -> bool {
    (self.data[x] & (1u64 << y)) != 0
  }

  #[inline(always)]
  pub fn shift_left(&self) -> Self {
    Self {
      data: std::array::from_fn(|i| {
        if i < Board::WIDTH - 1 {
          unsafe { *self.data.get_unchecked(i + 1) }
        } else {
          0
        }
      }),
    }
  }

  #[inline(always)]
  pub fn shift_right(&self) -> Self {
    Self {
      data: std::array::from_fn(|i| {
        if i == 0 {
          0
        } else {
          unsafe { *self.data.get_unchecked(i - 1) }
        }
      }),
    }
  }

  #[inline(always)]
  pub fn shift_down(&self) -> Self {
    Self {
      data: self.data.map(|c| c >> 1),
    }
  }

  #[inline(always)]
  pub fn shift_up(&self) -> Self {
    Self {
      data: self.data.map(|c| c << 1),
    }
  }

  #[inline(always)]
  pub fn shift(&self, dx: i8, dy: i8) -> Self {
    let mut board = Self::new();
    let adx = dx.unsigned_abs() as usize;
    if adx >= Self::WIDTH {
      return board;
    }

    let limit = Self::WIDTH - adx;

    for i in 0..limit {
      let src_idx = if dx < 0 { i + adx } else { i };
      let dest_idx = if dx >= 0 { i + adx } else { i };
      let v = unsafe { *self.data.get_unchecked(src_idx) };

      unsafe {
        *board.data.get_unchecked_mut(dest_idx) = if dy > 0 {
          v << dy
        } else if dy < 0 {
          v >> -dy
        } else {
          v
        };
      }
    }

    board
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.data.iter().all(|&x| x == 0)
  }

  #[inline(always)]
  pub fn real_height(&self) -> u32 {
    let mut tmp = 0;
    for x in 0..Self::WIDTH {
      tmp |= self.data[x];
    }
    64 - tmp.leading_zeros()
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

  #[inline(always)]
  pub fn count_ones(&self) -> u32 {
    let mut sum = 0;
    for x in 0..Self::WIDTH {
      sum += self.data[x].count_ones();
    }
    sum
  }

  #[inline(always)]
  pub fn line_clears(&self) -> u64 {
    let mut acc = !0u64;
    for x in 0..Self::WIDTH {
      acc &= self.data[x];
    }
    acc
  }

  #[inline(always)]
  fn clear_setup(&mut self, garbage_level: u8) -> (u64, u8) {
    let clear_mask = self.line_clears();

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
    let (mut clear_mask, garbage_cleared) = self.clear_setup(garbage_level);

    if clear_mask == 0 {
      return (0, 0);
    }

    let cleared_count = clear_mask.count_ones() as u8;

    loop {
      let mask = !(((clear_mask as i64 & -(clear_mask as i64)) - 1) as u64);
      for x in 0..Self::WIDTH {
        let col = self.data[x];
        self.data[x] = col ^ ((col ^ (col >> 1)) & mask);
      }
      clear_mask = (clear_mask & (clear_mask - 1)) >> 1;
      if clear_mask == 0 {
        break;
      }
    }

    (cleared_count, garbage_cleared)
  }
}

use std::ops::{
  BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
  ShrAssign,
};

impl BitOrAssign for Board {
  #[inline(always)]
  fn bitor_assign(&mut self, rhs: Self) {
    for (lhs, rhs) in self.data.iter_mut().zip(rhs.data.iter()) {
      *lhs |= *rhs;
    }
  }
}

impl BitOr for Board {
  type Output = Self;
  #[inline(always)]
  fn bitor(mut self, rhs: Self) -> Self::Output {
    self |= rhs;
    self
  }
}

impl BitAndAssign for Board {
  #[inline(always)]
  fn bitand_assign(&mut self, rhs: Self) {
    for (lhs, rhs) in self.data.iter_mut().zip(rhs.data.iter()) {
      *lhs &= *rhs;
    }
  }
}

impl BitAnd for Board {
  type Output = Self;
  #[inline(always)]
  fn bitand(mut self, rhs: Self) -> Self::Output {
    self &= rhs;
    self
  }
}

impl BitXorAssign for Board {
  #[inline(always)]
  fn bitxor_assign(&mut self, rhs: Self) {
    for (lhs, rhs) in self.data.iter_mut().zip(rhs.data.iter()) {
      *lhs ^= *rhs;
    }
  }
}

impl BitXor for Board {
  type Output = Self;
  #[inline(always)]
  fn bitxor(mut self, rhs: Self) -> Self::Output {
    self |= rhs; // Leverages register reuse
    self
  }
}

impl Not for Board {
  type Output = Self;
  #[inline(always)]
  fn not(mut self) -> Self::Output {
    for val in self.data.iter_mut() {
      *val = !*val;
    }
    self
  }
}

impl ShlAssign<usize> for Board {
  #[inline(always)]
  fn shl_assign(&mut self, rhs: usize) {
    for val in self.data.iter_mut() {
      *val <<= rhs;
    }
  }
}

impl Shl<usize> for Board {
  type Output = Self;
  #[inline(always)]
  fn shl(mut self, rhs: usize) -> Self::Output {
    self <<= rhs;
    self
  }
}

impl ShrAssign<usize> for Board {
  #[inline(always)]
  fn shr_assign(&mut self, rhs: usize) {
    for val in self.data.iter_mut() {
      *val >>= rhs;
    }
  }
}

impl Shr<usize> for Board {
  type Output = Self;
  #[inline(always)]
  fn shr(mut self, rhs: usize) -> Self::Output {
    self >>= rhs;
    self
  }
}

impl PartialEq for Board {
  fn eq(&self, other: &Self) -> bool {
    self.data == other.data
  }
}

impl Eq for Board {}

use std::ops::{Index, IndexMut};

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
