use std::collections::VecDeque;

mod data;
use data::{KickTable, Mino, get_kick_data, get_tetromino_data};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 40;
const BOARD_BUFFER: usize = 20;

const FULL_MASK: u16 = 0b1111111111;
const GARBAGE_MASK: u16 = 1 << 10;

pub struct Board {
  // Each element is a row where bits 0 to 9 are used. Row 10 is whether or not the row is made up of garbage.
  rows: [u16; BOARD_HEIGHT],
}

impl Board {
  pub fn new() -> Self {
    Board {
      rows: [0; BOARD_HEIGHT],
    }
  }

  // Set a cell to occupied.
  fn set(&mut self, x: usize, y: usize) {
    assert!(x < BOARD_WIDTH && y < BOARD_HEIGHT);
    self.rows[y] |= 1 << x;
  }

  // Check if a cell is occupied.
  fn is_occupied(&self, x: i8, y: i8) -> bool {
    if x < 0 || x >= BOARD_WIDTH as i8 || y < 0 || y >= BOARD_HEIGHT as i8 {
      return true;
    }
    self.rows[y as usize] & (1 << x as usize) != 0
  }

  fn clear(&mut self) -> (u8, bool) {
    let mut write = 0;
    let mut cleared: u8 = 0;
    let mut garbage_cleared = false;

    for read in 0..BOARD_HEIGHT {
      let row = self.rows[read];
      if (row & FULL_MASK) == FULL_MASK {
        cleared += 1;

        if row & GARBAGE_MASK != 0 {
          garbage_cleared = true;
        }
      } else {
        self.rows[write] = row;
        write += 1;
      }
    }
    for i in write..BOARD_HEIGHT {
      self.rows[i] = 0;
    }

    (cleared, garbage_cleared)
  }
}

type Queue = VecDeque<Mino>;

#[derive(Clone, Copy)]
pub struct Falling {
  x: u8,
  y: u8,
  rot: u8,
  piece: Mino,
}

pub struct Game {
  board: Board,
  queue: Queue,
  b2b: u32,
  combo: u32,
  hold: Option<Mino>,
  piece: Falling,
}

impl Game {
  pub fn new(queue: Vec<Mino>) -> Self {
    let tetromino = get_tetromino_data(queue[0]);
    Game {
      b2b: 0,
      combo: 0,
      board: Board::new(),
      queue: VecDeque::from_iter(queue[1..].iter().cloned()),
      hold: None,
      piece: Falling {
        x: ((BOARD_WIDTH as f32 / 2.0) - (tetromino.w as f32 / 2.0)) as u8,
        y: (BOARD_HEIGHT - BOARD_BUFFER) as u8,
        rot: 0,
        piece: queue[0],
      },
    }
  }

  pub fn rotate(&mut self, amount: u8) -> (bool, i8) {
    let target = (self.piece.rot + amount) % 4;
    let tetromino = get_tetromino_data(self.piece.piece);
    let target_data = &tetromino.data[target as usize];

    let piece_x = self.piece.x as i8;
    let piece_y = self.piece.y as i8;

    let mut collision = false;
    for &(x, y) in target_data {
      let nx = piece_x + x as i8;
      let ny = piece_y - y as i8;
      if self.board.is_occupied(nx, ny) {
        collision = true;
        break;
      }
    }

    if !collision {
      self.piece.rot = target;
      return (true, -1);
    }

    let kickset = get_kick_data(
      self.piece.piece,
      data::KickTable::SRSPlus,
      self.piece.rot,
      target,
    );

    for (index, &(dx, dy)) in kickset.iter().enumerate() {
      let mut valid = true;
      for &(x, y) in target_data {
        let nx = piece_x + x as i8 + dx as i8;
        let ny = piece_y - y as i8 - dy as i8;
        if self.board.is_occupied(nx, ny) {
          valid = false;
          break;
        }
      }

      if valid {
        self.piece.x = (piece_x + dx as i8) as u8;
        self.piece.y = (piece_y - dy as i8) as u8;
        self.piece.rot = target;
        return (true, index as i8);
      }
    }

    (false, -1)
  }

  pub fn move_left(&mut self) -> bool {
    let piece_x = self.piece.x as i8;
    let piece_y = self.piece.y as i8;

    for &(x, y) in get_tetromino_data(self.piece.piece).data[self.piece.rot as usize].iter() {
      if self
        .board
        .is_occupied(piece_x + x as i8 - 1, piece_y - y as i8)
      {
        return false;
      }
    }

    self.piece.x -= 1;
    true
  }

  pub fn move_right(&mut self) -> bool {
    let piece_x = self.piece.x as i8;
    let piece_y = self.piece.y as i8;

    for &(x, y) in get_tetromino_data(self.piece.piece).data[self.piece.rot as usize].iter() {
      if self
        .board
        .is_occupied(piece_x + x as i8 + 1, piece_y - y as i8)
      {
        return false;
      }
    }

    self.piece.x += 1;
    true
  }

  pub fn softdrop(&mut self) -> bool {
    let piece_x = self.piece.x as i8;
    let piece_y = self.piece.y as i8;
    let mut moved = false;

    let blocks = get_tetromino_data(self.piece.piece).data[self.piece.rot as usize];

    loop {
      for &(x, y) in blocks.iter() {
        if self
          .board
          .is_occupied(piece_x + x as i8, piece_y - y as i8 - 1)
        {
          return moved;
        }
      }
      moved = true;
    }
  }

  pub fn harddrop(&mut self) -> bool {
    self.softdrop();

    for &(x, y) in get_tetromino_data(self.piece.piece).data[self.piece.rot as usize].iter() {
      self
        .board
        .set(self.piece.x as usize + x as usize, self.piece.y as usize - y as usize);
    }
    
		let (cleared, garbage_cleared) = self.board.clear();

    true
  }
}
