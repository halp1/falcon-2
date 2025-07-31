use std::collections::VecDeque;

pub mod data;
use data::{ComboTable, KickTable, Mino, Spin, Spins};
use garbage::damage_calc;
use serde::Deserialize;

mod garbage;
pub mod queue;
pub mod rng;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 40;
pub const BOARD_BUFFER: usize = 20;

pub const BOARD_UPPER_HALF: usize = BOARD_HEIGHT / 2;
pub const BOARD_UPPER_QUARTER: usize = BOARD_HEIGHT / 4 * 3;

pub const CENTER_4: std::ops::Range<usize> = (BOARD_WIDTH / 2 - 2)..(BOARD_WIDTH / 2 + 2);
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

impl CollisionMap {
  fn new(board: &[u64; BOARD_WIDTH], piece: &Falling) -> CollisionMap {
    let mut states = [[0u64; BOARD_WIDTH + 2]; 4];

    for rot in 0usize..4usize {
      for (dx, dy) in piece.mino.rot(rot as u8) {
        let dx = *dx as usize;
        for x in 0..BOARD_WIDTH + 2 {
          let col = if x >= dx && x - dx < BOARD_WIDTH {
            board.get(x - dx).copied().unwrap_or(!0u64)
          } else {
            !0u64
          };
          states[rot][x] |= !(!col << dy);
        }
      }
    }

    CollisionMap { states }
  }

  pub fn test(&self, x: u8, y: u8, rot: u8) -> bool {
    let x = x as usize;
    let y = y as usize;
    if x >= BOARD_WIDTH + 2 || y >= BOARD_HEIGHT {
      return true;
    }
    (self.states[rot as usize][x] >> y) & 1 != 0
  }
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

  pub fn insert_garbage(&mut self, amount: u8, column: u8) {
    assert!((column as usize) < BOARD_WIDTH, "hole-column out of bounds");

    if amount == 0 {
      return;
    }

    self.garbage = (self.garbage.saturating_add(amount)).min(BOARD_HEIGHT as u8) as u8;

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

  pub fn max_height(&self) -> i32 {
    let mut max_h: i32 = 0;
    for &col in &self.cols {
      let h = (64u32 - col.leading_zeros()) as i32;
      if h > max_h {
        max_h = h;
      }
    }
    max_h
  }

  pub fn upper_half_height(&self) -> i32 {
    return (self.max_height() - BOARD_UPPER_HALF as i32).max(0);
  }

  pub fn upper_quarter_height(&self) -> i32 {
    return (self.max_height() - BOARD_UPPER_QUARTER as i32).max(0);
  }

  pub fn center_height(&self) -> i32 {
    let mut max_h: i32 = 0;
    for x in CENTER_4 {
      let col = self.cols[x];
      let h = (64u32 - col.leading_zeros()) as i32;
      if h > max_h {
        max_h = h;
      }
    }
    max_h
  }

  pub fn count_holes(&self) -> i32 {
    self
      .cols
      .iter()
      .map(|&col| (!col & ((1 << (64 - col.leading_zeros())) - 1)).count_ones())
      .sum::<u32>() as i32
  }

  pub fn unevenness(&self) -> i32 {
    let mut unevenness = 0;
    let mut last = 64 - self.cols[0].leading_zeros();

    for col in self.cols.iter().skip(1) {
      let h = 64 - col.leading_zeros();
      unevenness += (last as i32 - h as i32).abs();
      last = h;
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
  pub spins: Spins,
  pub b2b_charging: bool,
  pub b2b_charge_at: i16,
  pub b2b_charge_base: i16,
  pub b2b_chaining: bool,
  pub combo_table: ComboTable,
  pub garbage_multiplier: f32,
  pub pc_b2b: u16,
  pub pc_send: u16,
  pub garbage_special_bonus: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug)]
pub struct Garbage {
  pub col: u8,
  pub amt: u8,
  pub time: u8,
}

#[derive(Clone, Debug)]
pub struct Game {
  pub board: Board,
  pub queue: [Mino; 32],
  pub queue_ptr: usize,
  pub b2b: i16,
  pub combo: i16,
  pub hold: Option<Mino>,
  pub piece: Falling,
  pub garbage: VecDeque<Garbage>,
  pub collision_map: CollisionMap,
  pub spin: Spin,
}

impl Game {
  pub fn new(piece: Mino, queue: [Mino; 32]) -> Self {
    let tetromino = piece.data();
    let board = Board::new();
    let piece = Falling {
      x: ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1,
      y: (BOARD_HEIGHT - BOARD_BUFFER) as u8 + 2,
      rot: 0,
      mino: piece,
    };

    let collision_map = board.collision_map(&piece);

    Game {
      b2b: -1,
      combo: -1,
      board,
      queue,
      queue_ptr: 0,
      hold: None,
      piece,
      garbage: VecDeque::new(),
      collision_map,
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

  pub fn is_immobile(&self) -> bool {
    self
      .collision_map
      .test(self.piece.x, self.piece.y + 1, self.piece.rot)
      && self
        .collision_map
        .test(self.piece.x + 1, self.piece.y, self.piece.rot)
      && self
        .collision_map
        .test(self.piece.x, self.piece.y - 1, self.piece.rot)
      && self
        .collision_map
        .test(self.piece.x - 1, self.piece.y, self.piece.rot)
  }

  // Returns (success, spin, tst_or_fin)
  pub fn rotate(&mut self, amount: u8, config: &GameConfig) -> (bool, bool) {
    let to = (self.piece.rot + amount) % 4;

    let mut res = (false, false, false);

    if !self.collision_map.test(self.piece.x, self.piece.y, to) {
      self.piece.rot = to;
      res = (true, false, false);
    }

    if res.0 == false {
      let from = self.piece.rot;

      let kickset = KickTable::SRSPlus.data(self.piece.mino, from, to);

      for &(dx, dy) in kickset.iter() {
        if !self.collision_map.test(
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
      self.update_spin(res.2, config);
    }

    (res.0, res.1)
  }

  #[inline(always)]
  pub fn update_spin(&mut self, is_tst_or_fin: bool, config: &GameConfig) {
    if config.spins == Spins::None {
      return;
    }

    let t_status = if self.piece.mino == Mino::T {
      if is_tst_or_fin {
        Spin::Normal
      } else {
        self.detect_t_spin()
      }
    } else {
      Spin::None
    };

    if t_status != Spin::None
      || config.spins == Spins::T
      || config.spins == Spins::Mini
      || config.spins == Spins::All
    {
      self.spin = t_status;
      return;
    }

    let immobile = self.is_immobile();

    // if immobile {
    // 	println!("IMMOBILE SPIN FOUND FOR {} {} {} {}", self.piece.mino.str(), self.piece.x, self.piece.y, self.piece.rot);
    // 	self.board.print();
    // }

    if immobile {
      if self.piece.mino == Mino::T {
        self.spin = Spin::Mini;
      } else {
        self.spin = match config.spins {
          Spins::AllPlus | Spins::All => Spin::Normal,
          Spins::MiniPlus | Spins::Mini => Spin::Mini,
          _ => Spin::None,
        }
      }
    } else {
      self.spin = Spin::None;
    }
  }

  #[inline(always)]
  pub fn detect_t_spin(&self) -> Spin {
    let x = self.piece.x as i8;
    let y = self.piece.y as i8;

    let corners = [
      self.board.is_occupied(x - 2, y),
      self.board.is_occupied(x, y),
      self.board.is_occupied(x, y - 2),
      self.board.is_occupied(x - 2, y - 2),
    ];

    // println!("CORNER: {} {} {} {} {}", corners[0], corners[1], corners[2], corners[3], self.piece.mino.str());

    let mut corner_count = 0;
    for corner in corners {
      if corner {
        corner_count += 1;
      }
    }

    if corner_count < 3 {
      return Spin::None;
    }

    let rot = self.piece.rot as usize;

    if corners[rot] && corners[(rot + 1) % 4] {
      return Spin::Normal;
    }

    Spin::Mini
  }

  pub fn move_left(&mut self) -> bool {
    if self
      .collision_map
      .test(self.piece.x - 1, self.piece.y, self.piece.rot)
    {
      return false;
    }

    self.piece.x -= 1;
    true
  }

  pub fn move_right(&mut self) -> bool {
    if self
      .collision_map
      .test(self.piece.x + 1, self.piece.y, self.piece.rot)
    {
      return false;
    }

    self.piece.x += 1;
    true
  }

  pub fn das_right(&mut self) -> bool {
    let mut x = self.piece.x;
    let mut moved = false;
    while !self.collision_map.test(x + 1, self.piece.y, self.piece.rot) {
      moved = true;
      x += 1;
    }

    self.piece.x = x;

    moved
  }

  pub fn das_left(&mut self) -> bool {
    let mut x = self.piece.x;
    let mut moved = false;
    while !self.collision_map.test(x - 1, self.piece.y, self.piece.rot) {
      moved = true;
      x -= 1;
    }

    self.piece.x = x;

    moved
  }

  pub fn soft_drop(&mut self) -> bool {
    let piece_x = self.piece.x;
    let mut piece_y = self.piece.y;
    let mut moved = false;

    while piece_y > 0
      && !self
        .collision_map
        .test(piece_x, piece_y - 1, self.piece.rot)
    {
      moved = true;
      piece_y -= 1;
    }

    self.piece.y = piece_y;

    moved
  }

  pub fn hold(&mut self) -> bool {
    if let Some(hold) = self.hold {
      self.hold = Some(self.piece.mino);
      self.piece.mino = hold;
      let tetromino = self.piece.mino.data();
      self.piece.x = ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1;
      self.piece.y = (BOARD_HEIGHT - BOARD_BUFFER) as u8 + 2;
    } else {
      assert!(self.queue_ptr < self.queue.len(), "Queue is empty");
      self.hold = Some(self.piece.mino);
      self.next_piece();
    }

    true
  }

  pub fn next_piece(&mut self) {
    assert!(self.queue.len() > 0, "Queue is empty");
    let next = self.queue[self.queue_ptr];
    self.queue_ptr += 1;

    self.piece.mino = next;

    let tetromino = next.data();

    self.piece.x = ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1;
    self.piece.y = (BOARD_HEIGHT - BOARD_BUFFER) as u8 + 2;
    self.piece.rot = 0;
  }

  pub fn topped_out(&self) -> bool {
    self
      .collision_map
      .test(self.piece.x, self.piece.y, self.piece.rot)
  }

  pub fn regen_collision_map(&mut self) {
    self.collision_map = self.board.collision_map(&self.piece);
  }

  pub fn hard_drop(&mut self, config: &GameConfig) -> (u16, Option<Spin>) {
    // println!("HARD DROP {} {} {} {}", self.piece.mino.str(), self.piece.x, self.piece.y, self.piece.rot);
    self.soft_drop();

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
      sent += config.pc_send;
    }

    if let Some(b2b) = broke_b2b {
      if config.b2b_charging && b2b + 1 > config.b2b_charge_at {
        sent += ((b2b - config.b2b_charge_at + config.b2b_charge_base + 1) as f32
          * config.garbage_multiplier) as u16;
      }
    }

    if cleared > 0 {
      while sent > 0 && !self.garbage.is_empty() {
        let g = self.garbage.front_mut().unwrap();

        let g16 = g.amt as u16;

        if g16 > sent {
          g.amt -= sent as u8;
          sent = 0;
          break;
        } else {
          sent -= g16;
          self.garbage.pop_front();
        }
      }
    } else {
      while !self.garbage.is_empty() && self.garbage.front().unwrap().time == 0 {
        let g = self.garbage.pop_front().unwrap();
        self.board.insert_garbage(g.amt, g.col);
      }
    }

    for g in self.garbage.iter_mut() {
      if g.time > 0 {
        g.time -= 1;
      }
    }

    let clear_type = if cleared >= 4 {
      Option::Some(Spin::Normal)
    } else if cleared > 0 {
      Option::Some(self.spin)
    } else {
      None
    };

    self.spin = Spin::None;

    self.next_piece();

    (sent, clear_type)
  }
}
