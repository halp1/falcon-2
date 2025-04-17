use std::collections::VecDeque;

pub mod data;
use data::{KickTable, Mino};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 40;
const BOARD_BUFFER: usize = 20;

pub struct CollisionMap {
  states: [[u64; 10]; 4],
}

impl CollisionMap {
  fn new(board: &[u64; 10], piece: &Falling) -> CollisionMap {
    let mut states = [[0u64; 10]; 4];

    for rot in 0usize..4usize {
      for (dx, dy) in piece.mino.rot(piece.rot) {
        let dx = *dx as usize;
        for x in 0..BOARD_WIDTH {
          let col = if x >= dx {
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
  cols: [u64; BOARD_WIDTH],
  garbage: u8,
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

  pub fn clear(&mut self) -> (u16, bool) {
    let mut cleared = 0;
    let mut garbage_cleared = false;

    for y in BOARD_HEIGHT..0 {
      for x in 0..BOARD_WIDTH {
        if self.cols[x] & (1 << y) == 0 {
          break;
        }
        if x == BOARD_WIDTH - 1 {
          cleared += 1;
          if y < self.garbage as usize {
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

  #[cfg(debug_assertions)]
  pub fn print(&self) {
    let mut start_row = 0;
    for y in (0..BOARD_HEIGHT).rev() {
      let mut empty_row = true;
      for x in 0..BOARD_WIDTH {
        if (self.cols[x] & (1 << y)) != 0 {
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
    for _ in 0..BOARD_WIDTH {
      print!("--");
    }
    println!("+");
    for y in (0..=start_row).rev() {
      print!("|");
      for x in 0..BOARD_WIDTH {
        if (self.cols[x] & (1 << y)) != 0 {
          if y < self.garbage as usize {
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
    for _ in 0..BOARD_WIDTH {
      print!("--");
    }
    println!("+");
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

type Queue = VecDeque<Mino>;

#[derive(Clone, Copy)]
pub struct Falling {
  x: u8,
  y: u8,
  rot: u8,
  mino: Mino,
}

impl Falling {
  pub fn blocks(&self) -> &[(u8, u8); 4] {
    self.mino.rot(self.rot)
  }
}

pub struct Garbage {
  pub col: u8,
  pub amt: u8,
  pub time: u8,
}

pub struct Game {
  pub board: Board,
  pub queue: Queue,
  pub b2b: u32,
  pub combo: u32,
  pub hold: Option<Mino>,
  pub piece: Falling,
  pub garbage: VecDeque<Garbage>,
  pub collision_map: CollisionMap,
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
      b2b: 0,
      combo: 0,
      board,
      queue: VecDeque::from_iter(queue[1..].iter().cloned()),
      hold: None,
      piece,
      garbage: VecDeque::new(),
      collision_map,
    }
  }

  // Returns (success, spin, tst_or_fin)
  pub fn rotate(&mut self, amount: u8) -> (bool, bool, bool) {
    let to = (self.piece.rot + amount) % 4;
    let target_data = self.piece.mino.rot(to);

    let piece_x = self.piece.x;
    let piece_y = self.piece.y;

    let mut collision = false;
    for &(x, y) in target_data {
      let nx = piece_x - x;
      let ny = piece_y - y;
      if self.board.is_occupied(nx as i8, ny as i8) {
        collision = true;
        break;
      }
    }

    if !collision {
      self.piece.rot = to;
      return (true, false, false);
    }

    let from = self.piece.rot;

    let kickset = KickTable::SRSPlus.data(self.piece.mino, from, to);

    for &(dx, dy) in kickset.iter() {
      let mut valid = true;
      for &(x, y) in target_data {
        let nx = (piece_x - x) as i8 + dx;
        let ny = (piece_y - y) as i8 - dy;
        if self.board.is_occupied(nx, ny) {
          valid = false;
          break;
        }
      }

      if valid {
        let is_tst_or_fin =
          (((from == 2 && to == 3) || (from == 0 && to == 3)) && dx == 1 && dy == -2)
            || (((from == 2 && to == 1) || (from == 0 && to == 1)) && dx == -1 && dy == -2);
        self.piece.x = (piece_x as i8 - dx) as u8;
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

  pub fn next_piece(&mut self) -> bool {
    assert!(self.queue.len() > 0, "Queue is empty");
    let next = self.queue.pop_front().unwrap();

    self.piece.mino = next;

    let tetromino = next.data();

    self.piece.x = ((BOARD_WIDTH + tetromino.w as usize) / 2) as u8 - 1;
    self.piece.y = (BOARD_HEIGHT - BOARD_BUFFER) as u8;
    self.piece.rot = 0;

    self.collision_map = self.board.collision_map(&self.piece);

    self
      .collision_map
      .test(self.piece.x, self.piece.y, self.piece.rot)
  }

  pub fn hard_drop(&mut self) -> bool {
    self.soft_drop();

    println!("({} {})", self.piece.x, self.piece.y);
    let mut cb = Board::new();
    cb.cols = self.collision_map.states[self.piece.rot as usize].clone();
    cb.print();

    for &(x, y) in self.piece.blocks() {
      self
        .board
        .set((self.piece.x - x) as usize, (self.piece.y - y) as usize);
    }

    let (cleared, garbage_cleared) = self.board.clear();

    if cleared == 0 {
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

    self.next_piece();

    true
  }
}
