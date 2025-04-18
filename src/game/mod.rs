use std::collections::VecDeque;

pub mod data;
use data::{ComboTable, KickTable, Mino, Spin};
use garbage::damage_calc;

mod garbage;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 40;
pub const BOARD_BUFFER: usize = 20;

pub fn print_board(board: Vec<u64>, garbage_height: u8) {
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

  print!("+");
  for _ in 0..board.len() {
    print!("--");
  }
  println!("+");
  for y in (0..=start_row).rev() {
    print!("|");
    for col in board.iter() {
      if (col & (1 << y)) != 0 {
        if y < garbage_height as usize {
          print!("\x1b[47m  \x1b[0m");
        } else {
          print!("\x1b[41m  \x1b[0m");
        }
      } else {
        print!("  ");
      }
    }
    println!("|");
  }
  print!("+");
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
  fn new(board: &[u64; 10], piece: &Falling) -> CollisionMap {
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
    let res = self.states[rot as usize]
      .get(x as usize)
      .map(|c| c & (1 << y) != 0)
      .unwrap_or(true);

    res
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

  pub fn set(&mut self, x: usize, y: usize) {
    assert!(x < BOARD_WIDTH && y < BOARD_HEIGHT);
    self.cols[x] |= 1 << y;
  }

  fn is_occupied(&self, x: i8, y: i8) -> bool {
    if x < 0 || x >= BOARD_WIDTH as i8 || y < 0 || y >= BOARD_HEIGHT as i8 {
      return true;
    }
    let x = x as usize;
    let y = y as usize;
    (self.cols[x] & (1 << y)) != 0
  }

  pub fn clear(&mut self, max: u8) -> (u8, bool) {
    let mut cleared = 0;
    let mut garbage_cleared = false;

    for y in (0u8..max + 1).rev() {
      for x in 0..BOARD_WIDTH {
        if self.cols[x] & (1 << y) == 0 {
          break;
        }
        if x == BOARD_WIDTH - 1 {
          cleared += 1;

          if y < self.garbage {
            garbage_cleared = true;
            self.garbage -= 1;
          }

          for clear_x in 0..BOARD_WIDTH {
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
    print_board(Vec::from(self.cols), self.garbage);
  }

  pub fn collision_map(&self, piece: &Falling) -> CollisionMap {
    CollisionMap::new(&self.cols, piece)
  }

  // BOARD STATS

  pub fn max_height(&self) -> u8 {
    let mut max_height = 0;
    for x in 0..BOARD_WIDTH {
      let col = self.cols[x];
      if col != 0 {
        let highest_bit = 63 - col.leading_zeros() as u8;
        max_height = max_height.max(highest_bit + 1);
      }
    }
    max_height
  }

  pub fn center_height(&self) -> u8 {
    let mut total_height = 0;
    for x in (BOARD_WIDTH / 2 - 2)..=(BOARD_WIDTH / 2 + 2) {
      let col = self.cols[x];
      if col != 0 {
        let highest_bit = 63 - col.leading_zeros() as u8;
        total_height += highest_bit + 1;
      }
    }
    (total_height / BOARD_WIDTH as u8).max(1)
  }
}

type Queue = Vec<Mino>;

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

pub struct GameConfig {
  pub b2b_charging: bool,
  pub b2b_charge_at: i16,
  pub b2b_charge_base: i16,
  pub b2b_chaining: bool,
  pub combo_table: ComboTable,
  pub garbage_multiplier: f32,
  pub pc_b2b: u16,
  pub garbage_special_bonus: bool,
}

#[derive(Clone, Debug)]
pub struct Garbage {
  pub col: u8,
  pub amt: u8,
  pub time: u8,
}

#[derive(Clone, Debug)]
pub struct Game {
  pub board: Board,
  pub queue: VecDeque<Mino>,
  pub b2b: i16,
  pub combo: i16,
  pub hold: Option<Mino>,
  pub piece: Falling,
  pub garbage: VecDeque<Garbage>,
  pub collision_map: CollisionMap,
  pub spin: Spin,
}

impl Game {
  pub fn new(queue: Vec<Mino>) -> Self {
    assert!(queue.len() > 0, "Queue must contain at least one piece");
    let tetromino = queue[0].data();
    let board = Board::new();
    let piece = Falling {
      x: ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1,
      y: (BOARD_HEIGHT - BOARD_BUFFER) as u8,
      rot: 0,
      mino: queue[0],
    };

    let collision_map = board.collision_map(&piece);

    Game {
      b2b: -1,
      combo: -1,
      board,
      queue: VecDeque::from_iter(queue[1..].iter().cloned()),
      hold: None,
      piece,
      garbage: VecDeque::new(),
      collision_map,
      spin: Spin::None,
    }
  }

  // Returns (success, spin, tst_or_fin)
  pub fn rotate(&mut self, amount: u8) -> (bool, bool, bool) {
    let to = (self.piece.rot + amount) % 4;

    let piece_x = self.piece.x;
    let piece_y = self.piece.y;

    if !self.collision_map.test(
			self.piece.x,
			self.piece.y,
			to,
		) {
      self.piece.rot = to;
      return (true, false, false);
    }

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
        self.piece.x = (piece_x as i8 + dx) as u8;
        self.piece.y = (piece_y as i8 - dy) as u8;
        self.piece.rot = to;
        return (true, true, is_tst_or_fin);
      }
    }

    (false, false, false)
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

  pub fn hold(&mut self) {
    if let Some(hold) = self.hold {
      self.hold = Some(self.piece.mino);
      self.piece.mino = hold;
    } else {
      assert!(self.queue.len() > 0, "Queue is empty");
      self.hold = Some(self.piece.mino);
      self.next_piece();
    }
  }

  pub fn next_piece(&mut self) {
    assert!(self.queue.len() > 0, "Queue is empty");
    let next = self.queue.pop_front().unwrap();

    self.piece.mino = next;

    let tetromino = next.data();

    self.piece.x = ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1;
    self.piece.y = (BOARD_HEIGHT - BOARD_BUFFER) as u8;
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

  pub fn hard_drop(&mut self, config: &GameConfig) -> u16 {
    self.soft_drop();

    let blocks = self.piece.blocks();

    for &(x, y) in blocks {
      self
        .board
        .set((self.piece.x - x) as usize, (self.piece.y - y) as usize);
    }

    let (cleared, garbage_cleared) = self.board.clear(self.piece.y);

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

    self.spin = Spin::None;

    self.next_piece();

    sent
  }
}
