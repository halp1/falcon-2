#![feature(
  test_incomplete_feature,
  adt_const_params,
  generic_const_exprs,
  inherent_associated_types,
  mgca_type_const_syntax,
  const_index,
  const_trait_impl,
  const_slice_make_iter,
  generic_const_items,
  portable_simd
)]
use engine::game2::{
  board::Board,
  config::ConstConfig,
  data::{KickTable, Mino},
  map::CollisionMap,
  movegen::{expand},
};

fn test_piece<const PIECE: Mino>(board: &Board)
{
  let map = CollisionMap::usable::<PIECE>(board);

  println!("\nboard:");
  board.print();
  println!("\noriginal collisionmap:");
  map.print_cropped::<PIECE>(10);
  println!("\nlandable");
  map.landable().print_cropped::<PIECE>(10);
  println!("test");

  let result = expand::<
    PIECE,
    {
      ConstConfig {
        kicktable: KickTable::SRSPlus,
        enable_180: true,
      }
    },
  >(&map, (4, 21));

  let result_map = result[0];

  println!("result:");
  result_map.print_cropped::<{ Mino::T }>(10);

  assert!(false, "test");
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

  test_piece::<{ Mino::T }>(&board);
}
