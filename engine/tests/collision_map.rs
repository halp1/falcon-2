#![feature(min_adt_const_params)]

use engine::game2::{board::Board, data::Mino, map::CollisionMap};

fn raw_collision_map(&board: &Board, mino: Mino) -> CollisionMap {
  let mut cmap = CollisionMap::blank();
  // very slow version
  for rot in 0..mino.real_permutations() {
    for x in 0..Board::WIDTH as i8 {
      // cmap[rot][x as usize] |= (((1 << (u64::BITS - (&Board::HEIGHT - 5))) - 1) << (&Board::HEIGHT - 5));
      for y in 0..Board::HEIGHT as i8 {
        let blocks = mino.rot(rot as u8);
        if blocks
          .iter()
          .map(|block| (block.0 + x, block.1 + y))
          .all(|(x, y)| {
            x >= 0
              && (x as usize) < Board::WIDTH
              && y >= 0
              && (y as u32) < (&Board::HEIGHT + mino.data().w as u32)
              && board[x as usize] & (1 << y) == 0
          })
        {
          cmap[rot][x as usize] |= 1 << y;
        }
      }
    }
  }

  cmap
}

fn test_piece<const PIECE: Mino>(&board: &Board) {
  let mut a = raw_collision_map(&board, PIECE);
  let mut b = CollisionMap::usable::<PIECE>(&board);
  let mask = (1 << Board::HEIGHT) - 1;
  for rot in 0..4 {
    a[rot] &= Board {
      data: [mask; Board::WIDTH],
    };
		b[rot] &= Board {
      data: [mask; Board::WIDTH],
    };
  }

	assert_eq!(a, b);
}

#[test]
fn collision_map() {
  let mut board = Board::new();
  for (x, y) in [
    (0, 0),
    (1, 0),
    (3, 0),
    (4, 0),
    (5, 0),
    (6, 0),
    (9, 0),
    (0, 1),
    (4, 1),
    (5, 1),
    (8, 1),
    (9, 1),
    (0, 2),
    (3, 2),
    (4, 2),
    (8, 2),
  ] {
    board.set(x, y);
  }

  let mut a = raw_collision_map(&board, Mino::I);
  let b = CollisionMap::usable::<{ Mino::I }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::I }>();

	test_piece::<{ Mino::I }>(&board);

	
  let mut a = raw_collision_map(&board, Mino::J);
  let b = CollisionMap::usable::<{ Mino::J }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::J }>();

	test_piece::<{ Mino::J }>(&board);

	
  let mut a = raw_collision_map(&board, Mino::L);
  let b = CollisionMap::usable::<{ Mino::L }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::L }>();

	test_piece::<{ Mino::L }>(&board);

	
  let mut a = raw_collision_map(&board, Mino::Z);
  let b = CollisionMap::usable::<{ Mino::Z }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::Z }>();

	test_piece::<{ Mino::Z }>(&board);

	
  let mut a = raw_collision_map(&board, Mino::S);
  let b = CollisionMap::usable::<{ Mino::S }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::S }>();

	test_piece::<{ Mino::S }>(&board);

	
  let mut a = raw_collision_map(&board, Mino::T);
  let b = CollisionMap::usable::<{ Mino::T }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::T }>();

	test_piece::<{ Mino::T }>(&board);

	
  let mut a = raw_collision_map(&board, Mino::O);
  let b = CollisionMap::usable::<{ Mino::O }>(&board);
  a[2] = b[0];
  a[3] = b[1];

  a.print::<{ Mino::O }>();

	test_piece::<{ Mino::O }>(&board);
}
