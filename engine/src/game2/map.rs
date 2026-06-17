use super::board::Board;
use super::data::Mino;
use const_for::const_for;

pub const fn real_permutations<const PIECE: Mino>() -> usize {
  PIECE.real_permutations()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionMap {
  pub data: [Board; 4],
}

impl CollisionMap {
  const fn extract_rot_data<const PIECE: Mino>(
    rot: u8,
  ) -> (i8, i8, i8, i8, i8, i8, i8, i8, u64, u64, u64, u64) {
    let blocks = PIECE.rot(rot as u8);

    let mask0 = if blocks[0].1 < 0 {
      (1u64 << (-blocks[0].1) as usize) - 1
    } else {
      0
    };
    let mask1 = if blocks[1].1 < 0 {
      (1u64 << (-blocks[1].1) as usize) - 1
    } else {
      0
    };
    let mask2 = if blocks[2].1 < 0 {
      (1u64 << (-blocks[2].1) as usize) - 1
    } else {
      0
    };
    let mask3 = if blocks[3].1 < 0 {
      (1u64 << (-blocks[3].1) as usize) - 1
    } else {
      0
    };

    (
      blocks[0].0,
      blocks[0].1,
      blocks[1].0,
      blocks[1].1,
      blocks[2].0,
      blocks[2].1,
      blocks[3].0,
      blocks[3].1,
      mask0,
      mask1,
      mask2,
      mask3,
    )
  }

  pub const fn blank() -> Self {
    Self {
      data: [Board::new(); 4],
    }
  }

  pub const fn usable<const PIECE: Mino>(board: &Board) -> Self {
    let mut result = Self::blank();

    const MAX_SIZE: usize = {
      let minos = [
        Mino::I,
        Mino::J,
        Mino::O,
        Mino::S,
        Mino::T,
        Mino::Z,
        Mino::L,
      ];

      let mut max_w = 0;
      let mut i = 0;

      while i < minos.len() {
        let w = minos[i].data().w as usize;
        if w > max_w {
          max_w = w;
        }
        i += 1;
      }

      max_w - 1
    };

    let mut padded = [!0u64; MAX_SIZE + Board::WIDTH + 2];

    padded[MAX_SIZE..MAX_SIZE + Board::WIDTH].copy_from_slice(&board.data);

    macro_rules! make_rot {
      ($rot:expr) => {{
        let (dx0, dy0, dx1, dy1, dx2, dy2, dx3, dy3, mask0, mask1, mask2, mask3) =
          const { Self::extract_rot_data::<PIECE>($rot) };

        const_for!(x in 0..Board::WIDTH as i8 => {
          unsafe {
            let c0 = *padded.get_unchecked((MAX_SIZE as i8 + x + dx0) as usize);
            let c1 = *padded.get_unchecked((MAX_SIZE as i8 + x + dx1) as usize);
            let c2 = *padded.get_unchecked((MAX_SIZE as i8 + x + dx2) as usize);
            let c3 = *padded.get_unchecked((MAX_SIZE as i8 + x + dx3) as usize);

            let val0 = if dy0 >= 0 { c0 >> dy0 as u32 } else { (c0 << (-dy0) as u32) | mask0 };
            let val1 = if dy1 >= 0 { c1 >> dy1 as u32 } else { (c1 << (-dy1) as u32) | mask1 };
            let val2 = if dy2 >= 0 { c2 >> dy2 as u32 } else { (c2 << (-dy2) as u32) | mask2 };
            let val3 = if dy3 >= 0 { c3 >> dy3 as u32 } else { (c3 << (-dy3) as u32) | mask3 };

            result.data[$rot].data[x as usize] = !(val0 | val1 | val2 | val3) & ((1u64 << Board::HEIGHT) - 1);
          }
        });
      }};
    }

    match PIECE.real_permutations() {
      1 => {
        make_rot!(0);
      }
      2 => {
        make_rot!(0);
        make_rot!(1);
      }
      4 => {
        make_rot!(0);
        make_rot!(1);
        make_rot!(2);
        make_rot!(3);
      }
      _ => unreachable!(),
    };

    result
  }

  pub fn landable(&self) -> Self {
    CollisionMap {
      data: std::array::from_fn(|i| self.data[i] & !self.data[i].shift(0, 1)),
    }
  }

  pub fn print_cropped<const PIECE: Mino>(&self, crop_at: u32) {
    let max_h = self.data[0..PIECE.real_permutations()]
      .iter()
      .fold(0, |a, b| a.max(b.real_height()))
      .min(Board::HEIGHT)
      .min(crop_at);

    let boards = self.data[0..PIECE.real_permutations()]
      .iter()
      .map(|&board| board.str_vec(max_h))
      .collect::<Vec<_>>();

    for y in 0..max_h {
      for board in boards.iter().map(|b| b[y as usize].clone()) {
        print!("{}   ", board);
      }

      println!();
    }
  }

  pub fn print<const PIECE: Mino>(&self) {
    self.print_cropped::<PIECE>(u32::MAX);
  }

  pub fn count_ones(&self) -> u32 {
    self.data.iter().map(|b| b.count_ones()).sum()
  }

  #[inline(always)]
  pub fn for_each_filled<F>(&self, piece: Mino, mut f: F)
  where
    F: FnMut(u8, u8, u8), // yields (rot, x, y)
  {
    let num_rots = piece.real_permutations();
    for rot in 0..num_rots {
      for x in 0..Board::WIDTH {
        let mut col = unsafe { *self.data.get_unchecked(rot).data.get_unchecked(x) };
        while col != 0 {
          let y = col.trailing_zeros() as u8;
          f(rot as u8, x as u8, y);
          col &= col - 1; // Clears the lowest set bit
        }
      }
    }
  }

  #[inline(always)]
  pub fn iter_filled<const PIECE: Mino>(&self) -> FilledIter<'_> {
    FilledIter::new(self, PIECE.real_permutations())
  }
}

pub struct FilledIter<'a> {
  map: &'a CollisionMap,
  num_rots: usize,
  rot: usize,
  x: usize,
  current_col: u64,
}

impl<'a> FilledIter<'a> {
  #[inline(always)]
  pub fn new(map: &'a CollisionMap, num_rots: usize) -> Self {
    let current_col = if num_rots > 0 && Board::WIDTH > 0 {
      unsafe { *map.data.get_unchecked(0).data.get_unchecked(0) }
    } else {
      0
    };
    Self {
      map,
      num_rots,
      rot: 0,
      x: 0,
      current_col,
    }
  }
}

impl<'a> Iterator for FilledIter<'a> {
  type Item = (u8, u8, u8); // (rot, x, y)

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.current_col != 0 {
        let y = self.current_col.trailing_zeros() as u8;
        let item = (self.rot as u8, self.x as u8, y);
        self.current_col &= self.current_col - 1; // Clear lowest set bit
        return Some(item);
      }

      self.x += 1;
      if self.x >= Board::WIDTH {
        self.x = 0;
        self.rot += 1;
        if self.rot >= self.num_rots {
          return None;
        }
      }

      unsafe {
        self.current_col = *self
          .map
          .data
          .get_unchecked(self.rot)
          .data
          .get_unchecked(self.x);
      }
    }
  }
}

use std::ops::{Index, IndexMut};

impl Index<usize> for CollisionMap {
  type Output = Board;

  #[inline]
  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}

impl IndexMut<usize> for CollisionMap {
  #[inline]
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.data[index]
  }
}
