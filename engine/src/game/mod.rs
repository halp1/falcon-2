pub mod data;
use garbage::damage_calc;
use serde::{Deserialize, Serialize};
use triangle::{
  engine::{queue::Mino, utils::KickTable},
  types::game::{ComboTable, Spin, SpinBonuses},
};

use crate::game::{data::{KickTableData, MinoData}, queue::Bag};

mod garbage;
pub mod queue;
pub mod rng;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 40;
pub const BOARD_BUFFER: usize = 20;

pub const FULL_WIDTH: std::ops::Range<usize> = 0..BOARD_WIDTH;

pub fn print_board(board: Vec<u64>, garbage_height: u8, highlight: (Mino, Vec<(u8, u8)>)) {
  let mut start_row = 0;
  for y in (0..BOARD_HEIGHT).rev() {
    let mut empty_row = true;
    for col in board.iter() {
      if (col & (1 << y)) != 0 {
        empty_row = false;
        break;
      }
    }
    if !empty_row {
      start_row = y;
      break;
    }
  }

  print!("  +");
  for _ in 0..board.len() {
    print!("--");
  }
  println!("+");
  for y in (0..=start_row).rev() {
    print!("{:2}|", y + 1);
    for (x, col) in board.iter().enumerate() {
      if (col & (1 << y)) != 0 {
        if highlight.1.iter().any(|v| v.0 == x as u8 && v.1 == y as u8) {
          print!("{}", highlight.0.block_str());
        } else if y < garbage_height as usize {
          print!("\x1B[48;2;68;68;68m  \x1B[49m");
        } else {
          print!("\x1B[100m  \x1B[49m");
        }
      } else {
        print!("  ");
      }
    }
    println!("|");
  }
  print!("  +");
  for _ in 0..board.len() {
    print!("--");
  }
  println!("+");
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionMap {
  pub states: [[u64; BOARD_WIDTH + 2]; 4],
}

// impl CollisionMap {
//   fn new(board: &[u64; BOARD_WIDTH], piece: &Falling) -> CollisionMap {
//     let mut states = [[0u64; BOARD_WIDTH + 2]; 4];

//     for rot in 0usize..4usize {
//       for (dx, dy) in piece.mino.rot(rot as u8) {
//         let dx = *dx as usize;
//         for x in 0..BOARD_WIDTH + 2 {
//           let col = if x >= dx && x - dx < BOARD_WIDTH {
//             board.get(x - dx).copied().unwrap_or(!0u64)
//           } else {
//             !0u64
//           };
//           states[rot][x] |= !(!col << dy);
//         }
//       }
//     }

//     CollisionMap { states }
//   }

//   pub fn test(&self, x: u8, y: u8, rot: u8) -> bool {
//     let x = x as usize;
//     let y = y as usize;
//     if x >= BOARD_WIDTH + 2 || y >= BOARD_HEIGHT {
//       return true;
//     }
//     (self.states[rot as usize][x] >> y) & 1 != 0
//   }
// }

impl CollisionMap {
  #[inline(always)]
  pub fn new(board: &[u64; BOARD_WIDTH], piece: &Falling) -> CollisionMap {
    let mut states = [[0u64; BOARD_WIDTH + 2]; 4];

    // 1. Algorithmic Padding: Map out-of-bounds logic directly to memory offsets.
    // Placing the active board at index 8 leaves a safe buffer of !0u64 on both sides.
    let mut padded = [!0u64; 32];
    padded[8..8 + BOARD_WIDTH].copy_from_slice(board);

    for rot in 0..4 {
      let blocks = piece.mino.rot(rot as u8);

      // 2. High-Level Inversion: Extract all 4 block coordinates at once
      let (dx0, dy0) = (blocks[0].0 as usize, blocks[0].1 as usize);
      let (dx1, dy1) = (blocks[1].0 as usize, blocks[1].1 as usize);
      let (dx2, dy2) = (blocks[2].0 as usize, blocks[2].1 as usize);
      let (dx3, dy3) = (blocks[3].0 as usize, blocks[3].1 as usize);

      // Pre-calculate shift masks to avoid repeating math inside the unrolled sections
      let mask0 = (1u64 << dy0) - 1;
      let mask1 = (1u64 << dy1) - 1;
      let mask2 = (1u64 << dy2) - 1;
      let mask3 = (1u64 << dy3) - 1;

      // 3. Complete Loop Flattening: Compute columns explicitly without internal loops.
      // This mirrors your C++ inspiration by executing a flat, branchless pipeline.
      macro_rules! compute_column {
        ($x:expr) => {
          unsafe {
            // Parallel fetch from the padded board layout
            let c0 = *padded.get_unchecked(8 + $x - dx0);
            let c1 = *padded.get_unchecked(8 + $x - dx1);
            let c2 = *padded.get_unchecked(8 + $x - dx2);
            let c3 = *padded.get_unchecked(8 + $x - dx3);

            // Bitwise identity substitution for !(!col << dy) -> (col << dy) | mask
            states[rot][$x] = ((c0 << dy0) | mask0)
              | ((c1 << dy1) | mask1)
              | ((c2 << dy2) | mask2)
              | ((c3 << dy3) | mask3);
          }
        };
      }

      for x in 0..BOARD_WIDTH + 2 {
        compute_column!(x);
      }
    }

    CollisionMap { states }
  }

  #[inline(always)]
  pub fn test(&self, x: u8, y: u8, rot: u8) -> bool {
    let x = x as usize;
    let y = y as usize;
    if x >= BOARD_WIDTH + 2 || y >= BOARD_HEIGHT {
      return true;
    }
    (self.states[rot as usize][x] >> y) & 1 != 0
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HoleData<T> {
  pub holes: T,
  pub depth: T,
  pub accessible: T,
  pub inaccessible: T,
}

#[derive(Clone, Copy, Debug)]
pub struct Board {
  pub cols: [u64; BOARD_WIDTH],
  pub garbage: u8,
}

impl Board {
  pub fn new() -> Self {
    Board {
      cols: [0; BOARD_WIDTH],
      garbage: 0,
    }
  }

  #[inline(always)]
  pub fn set(&mut self, x: usize, y: usize) {
    debug_assert!(x < BOARD_WIDTH && y < BOARD_HEIGHT);
    self.cols[x] |= 1 << y;
  }

  pub fn is_occupied(&self, x: i8, y: i8) -> bool {
    if x < 0 || x >= BOARD_WIDTH as i8 || y < 0 || y >= BOARD_HEIGHT as i8 {
      return true;
    }
    let x = x as usize;
    let y = y as usize;
    (self.cols[x] & (1 << y)) != 0
  }

  pub fn clear(&mut self, from: u8, to: u8) -> (u8, bool) {
    let mut cleared = 0;
    let mut garbage_cleared = false;

    for y in (from..to + 1).rev() {
      for x in FULL_WIDTH {
        if self.cols[x] & (1 << y) == 0 {
          break;
        }
        if x == BOARD_WIDTH - 1 {
          cleared += 1;

          if y < self.garbage {
            garbage_cleared = true;
            self.garbage -= 1;
          }

          for clear_x in FULL_WIDTH {
            let low_mask = (1u64 << y) - 1;
            let low = self.cols[clear_x] & low_mask;

            let high = self.cols[clear_x] >> (y + 1);
            self.cols[clear_x] = (high << y) | low;
          }
        }
      }
    }

    (cleared, garbage_cleared)
  }

  // #[inline(always)]
  // fn clear_column(bits: u64, to_clear: u64) -> u64 {
  //   let mut out = 0u64;
  //   let mut dst = 0;
  //   for src in 0..BOARD_HEIGHT {
  //     let m = 1u64 << src;
  //     if to_clear & m == 0 {
  //       // keep this row
  //       if bits & m != 0 {
  //         out |= 1u64 << dst;
  //       }
  //       dst += 1;
  //     }
  //   }
  //   out
  // }

  // pub fn clear(&mut self, from: u8, to: u8) -> (u8, bool) {
  //   let full_rows_mask = self.cols.iter().copied().fold(!0u64, |acc, col| acc & col);

  //   let window_mask = ((1u64 << (to - from + 1)) - 1) << from;
  //   let rows_to_clear = (full_rows_mask & window_mask) >> from;

  //   let cleared = rows_to_clear.count_ones() as u8;
  //   if cleared == 0 {
  //     return (0, false);
  //   }

  //   let garbage_range_mask = (1u64 << self.garbage) - 1;
  //   let garbage_bits = rows_to_clear & garbage_range_mask;
  //   let garbage_cleared = garbage_bits != 0;

  //   self.garbage -= garbage_bits.count_ones() as u8;

  //   for col in &mut self.cols {
  //     *col = Board::clear_column(*col, rows_to_clear);
  //   }

  //   (cleared, garbage_cleared)
  // }

  pub fn is_pc(&self) -> bool {
    for col in self.cols {
      if col & (1 << (BOARD_HEIGHT - 1)) != 0 {
        return true;
      }
    }

    false
  }

  pub fn insert_garbage(&mut self, amount: u16, column: u8) {
    assert!((column as usize) < BOARD_WIDTH, "hole-column out of bounds");

    if amount == 0 {
      return;
    }

    self.garbage = (self.garbage.saturating_add(amount as u8)).min(BOARD_HEIGHT as u8);

    let all_mask = (1u64 << BOARD_HEIGHT) - 1;
    let bottom_mask = (1u64 << amount) - 1;

    for x in 0..BOARD_WIDTH {
      let shifted = (self.cols[x] << amount) & all_mask;

      self.cols[x] = if x == column as usize {
        shifted
      } else {
        shifted | bottom_mask
      };
    }
  }

  pub fn print(&self) {
    print_board(Vec::from(self.cols), self.garbage, (Mino::I, Vec::new()));
  }

  pub fn collision_map(&self, piece: &Falling) -> CollisionMap {
    CollisionMap::new(&self.cols, piece)
  }

  // BOARD STATS

  #[inline(always)]
  pub fn column_heights(&self) -> [u32; BOARD_WIDTH] {
    std::array::from_fn(|i| 64 - self.cols[i].leading_zeros())
  }

  #[inline(always)]
  pub fn heights(&self) -> (u32, u32) {
    (
      64 - self.cols[..(BOARD_WIDTH - 4) / 2]
        .iter()
        .chain(self.cols[(BOARD_WIDTH - 3)..].iter())
        .fold(0, |acc, &val| acc | val)
        .leading_zeros(),
      64 - self.cols[(BOARD_WIDTH - 4) / 2..(BOARD_WIDTH - 3)]
        .iter()
        .fold(0, |acc, &val| acc | val)
        .leading_zeros(),
    )
  }

  #[inline(always)]
  pub fn well(&self, heights: &[u32; BOARD_WIDTH]) -> Option<usize> {
    let mut min1_val = heights[0];
    let mut min1_idx = 0;
    let mut min2_val = u32::MAX;

    for i in 1..BOARD_WIDTH {
      let h = heights[i];

      if h < min1_val {
        min2_val = min1_val;
        min1_val = h;
        min1_idx = i;
      } else if h < min2_val {
        min2_val = h;
      }
    }

    if min1_val < min2_val {
      Some(min1_idx)
    } else {
      None
    }
  }
  #[inline(always)]
  pub fn holes(&self, heights: &[u32; BOARD_WIDTH]) -> HoleData<u32> {
    let mut total_holes = 0;
    let mut summed_depth = 0;
    let mut accessible = 0;
    let mut inaccessible = 0;

    for x in 0..BOARD_WIDTH {
      let col = self.cols[x];
      let hole_mask = !col & ((1 << heights[x]) - 1);

      if hole_mask == 0 {
        continue;
      }

      let col_holes = hole_mask.count_ones();
      total_holes += col_holes;

      // check 3 blocks above the hole mask to determine accessibility
      summed_depth += (col & (hole_mask << 1)).count_ones()
        + (col & (hole_mask << 2)).count_ones()
        + (col & (hole_mask << 3)).count_ones();

      let activation_y = (if x < BOARD_WIDTH - 2 {
        std::cmp::max(heights[x + 1], heights[x + 2].saturating_sub(2))
      } else {
        u32::MAX
      })
      .min(if x >= 2 {
        std::cmp::max(heights[x - 1], heights[x - 2].saturating_sub(2))
      } else {
        u32::MAX
      });

      let activation_mask = !0u64 << activation_y;

      let col_accessible = (hole_mask & activation_mask).count_ones();

      accessible += col_accessible;
      inaccessible += col_holes - col_accessible;
    }

    HoleData {
      holes: total_holes,
      depth: summed_depth,
      accessible,
      inaccessible,
    }
  }

  pub fn count_holes(&self) -> i32 {
    self
      .cols
      .iter()
      .map(|&col| (!col & ((1 << (64 - col.leading_zeros())) - 1)).count_ones())
      .sum::<u32>() as i32
  }

  #[inline(always)]
  pub fn unevenness(&self, heights: &[u32; BOARD_WIDTH], well: Option<usize>) -> i32 {
    let mut unevenness = 0;
    let mut last = heights[0] as i32;

    for (i, &h) in heights.iter().skip(1).enumerate() {
      if well.map_or(false, |w| w == i) {
        continue;
      }
      unevenness += (last - h as i32).abs();
      last = h as i32;
    }

    unevenness
  }

  pub fn covered_holes(&self) -> i32 {
    self
      .cols
      .iter()
      .enumerate()
      .map(|(x, &col)| {
        let hole_map = !col & ((1 << (64 - col.leading_zeros())) - 1);

        (hole_map
          & (if x == 0 { !0u64 } else { self.cols[x - 1] })
          & (if x == BOARD_WIDTH - 1 {
            !0u64
          } else {
            self.cols[x + 1]
          }))
        .count_ones()
      })
      .sum::<u32>() as i32
  }

  pub fn overstacked_holes(&self) -> i32 {
    self
      .cols
      .iter()
      .map(|&col| {
        let mask = !col & (col >> 1);

        if mask == 0 {
          return 0;
        }

        ((63 - col.leading_zeros()) - mask.trailing_zeros() - 1).max(0)
      })
      .sum::<u32>() as i32
  }

  pub fn wells(&self) -> i32 {
    let heights = self
      .cols
      .iter()
      .map(|&col| 64 - col.leading_zeros())
      .collect::<Vec<u32>>();

    (heights
      .iter()
      .enumerate()
      .map(|(index, &height)| {
        if ((if index == 0 { 63 } else { heights[index - 1] })
          .min(if index == BOARD_WIDTH - 1 {
            63
          } else {
            heights[index + 1]
          })
          .saturating_sub(height))
          >= 3
        {
          1
        } else {
          0
        }
      })
      .sum::<u32>() as i32
      - 1)
      .max(0)
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Falling {
  pub x: u8,
  pub y: u8,
  pub rot: u8,
  pub mino: Mino,
}

impl Falling {
  pub fn blocks(&self) -> &[(u8, u8); 4] {
    self.mino.rot(self.rot)
  }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
  pub kicks: KickTable,
  pub spins: SpinBonuses,
  pub b2b_charging: bool,
  pub b2b_charge_at: i16,
  pub b2b_charge_base: i16,
  pub b2b_chaining: bool,
  pub combo_table: ComboTable,
  pub garbage_multiplier: f32,
  pub garbage_cap: u16,
  pub pc_b2b: u16,
  pub pc_send: u8,
  pub garbage_special_bonus: bool,
	pub bag: Bag,
}

pub struct StartState<'a> {
  pub queue: &'a [Mino; 32],
  pub garbage: &'a [Garbage],
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug)]
pub struct Garbage {
  pub col: u8,
  pub amt: u16,
  pub time: u8,
}

#[derive(Clone, Debug)]
pub struct Game {
  pub board: Board,
  pub queue_ptr: usize,
  pub b2b: i16,
  pub combo: i16,
  pub hold: Option<Mino>,
  pub piece: Falling,
  // index, tanked
  pub garbage: (usize, u16),
  pub spin: Spin,
}

impl Game {
  pub fn new(piece: Mino) -> Self {
    let tetromino = piece.data();
    let board = Board::new();
    let piece = Falling {
      x: ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1,
      y: (BOARD_HEIGHT - BOARD_BUFFER) as u8 + 2,
      rot: 0,
      mino: piece,
    };

    Game {
      b2b: -1,
      combo: -1,
      board,
      queue_ptr: 0,
      hold: None,
      piece,
      garbage: (0, 0),
      spin: Spin::None,
    }
  }

  pub fn print(&self) {
    let mut b = self.board.clone();
    let mut falling_target = Vec::new();
    for &(x, y) in self.piece.blocks() {
      if self.piece.x < x || self.piece.y < y {
        continue;
      }
      b.set((self.piece.x - x) as usize, (self.piece.y - y) as usize);
      falling_target.push((self.piece.x - x, self.piece.y - y));
    }

    print_board(
      Vec::from(b.cols),
      b.garbage,
      (self.piece.mino, falling_target),
    );
  }

  pub fn is_immobile(&self, collision_map: &CollisionMap) -> bool {
    collision_map.test(self.piece.x, self.piece.y + 1, self.piece.rot)
      && collision_map.test(self.piece.x + 1, self.piece.y, self.piece.rot)
      && collision_map.test(self.piece.x, self.piece.y - 1, self.piece.rot)
      && collision_map.test(self.piece.x - 1, self.piece.y, self.piece.rot)
  }

  // Returns (success, kicked)
  pub fn rotate(
    &mut self,
    amount: u8,
    config: &GameConfig,
    collision_map: &CollisionMap,
  ) -> (bool, bool) {
    let to = (self.piece.rot + amount) % 4;

    let mut res = (false, false, false);

    if !collision_map.test(self.piece.x, self.piece.y, to) {
      self.piece.rot = to;
      res = (true, false, false);
    }

    if res.0 == false {
      let from = self.piece.rot;

      let kickset = config.kicks.data_fast(self.piece.mino, from, to);

      for &(dx, dy) in kickset.iter() {
        if !collision_map.test(
          (self.piece.x as i8 + dx) as u8,
          (self.piece.y as i8 - dy) as u8,
          to,
        ) {
          let is_tst_or_fin =
            (((from == 2 && to == 3) || (from == 0 && to == 3)) && dx == 1 && dy == -2)
              || (((from == 2 && to == 1) || (from == 0 && to == 1)) && dx == -1 && dy == -2);
          self.piece.x = (self.piece.x as i8 + dx) as u8;
          self.piece.y = (self.piece.y as i8 - dy) as u8;
          self.piece.rot = to;
          res = (true, true, is_tst_or_fin);
          break;
        }
      }
    }

    if res.0 {
      self.update_spin(res.2, config, collision_map);
    }

    (res.0, res.1)
  }

  #[inline(always)]
  pub fn update_spin(
    &mut self,
    is_tst_or_fin: bool,
    config: &GameConfig,
    collision_map: &CollisionMap,
  ) {
    if config.spins == SpinBonuses::None {
      return;
    }

    let t_status = if self.piece.mino == Mino::T {
      self.detect_spin(is_tst_or_fin, collision_map)
    } else {
      Spin::None
    };

    let immobile = match config.spins {
      SpinBonuses::All
      | SpinBonuses::AllPlus
      | SpinBonuses::AllMini
      | SpinBonuses::AllMiniPlus
      | SpinBonuses::MiniOnly => self.is_immobile(collision_map),
      _ => false,
    };

    self.spin = match config.spins {
      SpinBonuses::None => Spin::None,
      SpinBonuses::Stupid => {
        if collision_map.test(self.piece.x, self.piece.y - 1, self.piece.rot) {
          Spin::Normal
        } else {
          Spin::None
        }
      }
      SpinBonuses::TSpins => t_status,
      SpinBonuses::TSpinsPlus => {
        if t_status != Spin::None {
          t_status
        } else if immobile && self.piece.mino == Mino::T {
          Spin::Mini
        } else {
          Spin::None
        }
      }
      SpinBonuses::All => {
        if self.piece.mino == Mino::T {
          t_status
        } else if immobile {
          Spin::Normal
        } else {
          Spin::None
        }
      }
      SpinBonuses::AllMini => {
        if self.piece.mino == Mino::T {
          t_status
        } else if immobile {
          Spin::Mini
        } else {
          Spin::None
        }
      }
      SpinBonuses::AllPlus => {
        if self.piece.mino == Mino::T {
          if t_status != Spin::None {
            t_status
          } else if immobile {
            Spin::Mini
          } else {
            Spin::None
          }
        } else {
          if immobile { Spin::Normal } else { Spin::None }
        }
      }
      SpinBonuses::AllMiniPlus => {
        if self.piece.mino == Mino::T {
          if t_status != Spin::None {
            t_status
          } else if immobile {
            Spin::Mini
          } else {
            Spin::None
          }
        } else {
          if immobile { Spin::Mini } else { Spin::None }
        }
      }
      SpinBonuses::MiniOnly => {
        if t_status != Spin::None {
          Spin::Mini
        } else if immobile {
          Spin::Mini
        } else {
          Spin::None
        }
      }
      SpinBonuses::Handheld => self.detect_spin(is_tst_or_fin, collision_map),
    };
  }

  #[inline(always)]
  pub fn detect_spin(&self, is_tst_or_fin: bool, collision_map: &CollisionMap) -> Spin {
    let x = self.piece.x as i8;
    let y = self.piece.y as i8;

    let table = if let Some(corner_table) = self.piece.mino.corner_table(self.piece.rot) {
      corner_table
    } else {
      return Spin::None;
    };

    if !collision_map.test(x as u8, y as u8 - 1, self.piece.rot) {
      return Spin::None;
    }

    let mut corners = 0u8;
    let mut front_corners = 0u8;

    let table = table[self.piece.rot as usize];

    for i in 0..4 {
      if self
        .board
        .is_occupied(x - table[i].0.0 + 1, y - table[i].0.1 - 1)
      {
        corners += 1;
        if let Some((r1, r2)) = table[i].1
          && (self.piece.rot == r1 || self.piece.rot == r2)
        {
          front_corners += 1;
        }
      }
    }

    if corners < 3 {
      return Spin::None;
    }

    let mut spin = Spin::Normal;
    if self.piece.mino == Mino::T && front_corners != 2 {
      spin = Spin::Mini;
    }
    if is_tst_or_fin {
      spin = Spin::Normal;
    }

    spin
  }

  pub fn move_left(&mut self, collision_map: &CollisionMap) -> bool {
    if collision_map.test(self.piece.x - 1, self.piece.y, self.piece.rot) {
      return false;
    }

    self.piece.x -= 1;
    true
  }

  pub fn move_right(&mut self, collision_map: &CollisionMap) -> bool {
    if collision_map.test(self.piece.x + 1, self.piece.y, self.piece.rot) {
      return false;
    }

    self.piece.x += 1;
    true
  }

  pub fn das_right(&mut self, collision_map: &CollisionMap) -> bool {
    let mut x = self.piece.x;
    let mut moved = false;
    while !collision_map.test(x + 1, self.piece.y, self.piece.rot) {
      moved = true;
      x += 1;
    }

    self.piece.x = x;

    moved
  }

  pub fn das_left(&mut self, collision_map: &CollisionMap) -> bool {
    let mut x = self.piece.x;
    let mut moved = false;
    while !collision_map.test(x - 1, self.piece.y, self.piece.rot) {
      moved = true;
      x -= 1;
    }

    self.piece.x = x;

    moved
  }

  pub fn soft_drop(&mut self, collision_map: &CollisionMap) -> bool {
    let piece_x = self.piece.x;
    let mut piece_y = self.piece.y;
    let mut moved = false;

    while piece_y > 0 && !collision_map.test(piece_x, piece_y - 1, self.piece.rot) {
      moved = true;
      piece_y -= 1;
    }

    self.piece.y = piece_y;

    moved
  }

  pub fn hold(&mut self, start_state: &StartState) -> bool {
    if let Some(hold) = self.hold {
      self.hold = Some(self.piece.mino);
      self.piece.mino = hold;
      let tetromino = self.piece.mino.data();
      self.piece.x = ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1;
      self.piece.y = (BOARD_HEIGHT - BOARD_BUFFER) as u8 + 2;
    } else {
      assert!(self.queue_ptr < start_state.queue.len(), "Queue is empty");
      self.hold = Some(self.piece.mino);
      self.next_piece(start_state);
    }

    true
  }

  pub fn next_piece(&mut self, start_state: &StartState) {
    assert!(self.queue_ptr < start_state.queue.len(), "Queue is empty");
    let next = start_state.queue[self.queue_ptr];
    self.queue_ptr += 1;

    self.piece.mino = next;

    let tetromino = next.data();

    self.piece.x = ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1;
    self.piece.y = (BOARD_HEIGHT - BOARD_BUFFER) as u8 + 2;
    self.piece.rot = 0;
  }

  pub fn topped_out(&self, collision_map: &CollisionMap) -> bool {
    collision_map.test(self.piece.x, self.piece.y, self.piece.rot)
  }

  #[inline(always)]
  pub fn topped_out_raw(&self) -> bool {
    self.piece.blocks().iter().any(|&(x, y)| {
      self
        .board
        .is_occupied(self.piece.x as i8 - x as i8, self.piece.y as i8 - y as i8)
    })
  }

  pub fn collision_map(&self) -> CollisionMap {
    self.board.collision_map(&self.piece)
  }

  // Returns (attack, actual sent garbage, (clear type + lines cleared))
  pub fn hard_drop(
    &mut self,
    config: &GameConfig,
    collision_map: &CollisionMap,
    state: &StartState,
    timer: u8,
  ) -> (u16, u16, (Spin, u8)) {
    // println!("HARD DROP {} {} {} {}", self.piece.mino.str(), self.piece.x, self.piece.y, self.piece.rot);
    self.soft_drop(collision_map);

    let blocks = self.piece.blocks();

    let mut max_y = blocks[0].1;
    let mut min_y = blocks[0].1;

    for &(x, y) in blocks {
      if !(self.piece.x >= x) {
        println!(
          "{} {} {} {}",
          self.piece.x,
          self.piece.y,
          self.piece.rot,
          self.piece.mino.block_str()
        );
        for &(x, y) in blocks {
          println!("{} {}", x, y);
        }
      }
      assert!(
        self.piece.x >= x,
        "x fail: {} {} {}",
        self.piece.x,
        x,
        self.piece.mino.block_str()
      );
      assert!(
        self.piece.y >= y,
        "y fail: {} {} {}",
        self.piece.y,
        y,
        self.piece.mino.block_str()
      );
      self
        .board
        .set((self.piece.x - x) as usize, (self.piece.y - y) as usize);

      if y > max_y {
        max_y = y;
      }
      if y < min_y {
        min_y = y;
      }
    }

    let (cleared, garbage_cleared) = self.board.clear(self.piece.y - max_y, self.piece.y - min_y);

    let pc = self.board.is_pc();

    let mut broke_b2b = Option::from(self.b2b);
    if cleared > 0 {
      self.combo += 1;
      if (self.spin != Spin::None || cleared >= 4) && !(pc && config.pc_b2b > 0) {
        self.b2b += 1;
        broke_b2b = None;
      }
      if pc && config.pc_b2b > 0 {
        self.b2b += config.pc_b2b as i16;
        broke_b2b = None;
      }

      if broke_b2b.is_some() {
        self.b2b = -1;
      }
    } else {
      self.combo = -1;
      broke_b2b = None;
    }

    let garbage_special_bonus = if config.garbage_special_bonus
      && garbage_cleared
      && (self.spin != Spin::None || cleared >= 4)
    {
      1
    } else {
      0
    } as f32;

    let mut sent = (damage_calc(
      cleared,
      self.spin,
      self.b2b,
      self.combo,
      config.combo_table,
      config.b2b_chaining,
    ) * config.garbage_multiplier
      + garbage_special_bonus) as u16;

    if pc {
      sent += config.pc_send as u16;
    }

    if let Some(b2b) = broke_b2b {
      if config.b2b_charging && b2b + 1 > config.b2b_charge_at {
        sent += ((b2b - config.b2b_charge_at + config.b2b_charge_base + 1) as f32
          * config.garbage_multiplier) as u16;
      }
    }

    let attack = sent;

    let gb_len = state.garbage.len();

    if cleared > 0 {
      while sent > 0 && self.garbage.0 < gb_len {
        let amt = state.garbage[self.garbage.0].amt;

        if amt > sent {
          self.garbage.1 = amt - sent;
          sent = 0;
          break;
        } else {
          sent -= amt;
          self.garbage.0 += 1;
          self.garbage.1 = 0;
        }
      }
    } else {
      let mut tanked = 0;
      while self.garbage.0 < gb_len
        && tanked < config.garbage_cap
        && state.garbage[self.garbage.0].time <= timer
      {
        let amt =
          (tanked - config.garbage_cap).min(state.garbage[self.garbage.0].amt - self.garbage.1);
        tanked += amt;
        self
          .board
          .insert_garbage(amt, state.garbage[self.garbage.0].col);

        if amt == state.garbage[self.garbage.0].amt - self.garbage.1 {
          self.garbage.0 += 1;
          self.garbage.1 = 0;
        } else {
          self.garbage.1 += amt;
        }
      }
    }

    let clear_type = self.spin;

    self.spin = Spin::None;

    self.next_piece(state);

    (attack, sent, (clear_type, cleared))
  }
}
