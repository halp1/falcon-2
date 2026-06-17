use criterion::{Criterion, black_box, criterion_group, criterion_main};

use engine::game2::{board::Board, data::Mino, map::CollisionMap};

fn collision_map(c: &mut Criterion) {
  println!(
    "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",
    Mino::I.data(),
    Mino::J.data(),
    Mino::L.data(),
    Mino::T.data(),
    Mino::S.data(),
    Mino::Z.data(),
    Mino::O.data(),
  );

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

  board.print();
  println!();

  CollisionMap::usable::<{ Mino::T }>(black_box(&board)).print::<{ Mino::T }>();

  for mino in [
    Mino::I,
    Mino::J,
    Mino::S,
    Mino::T,
    Mino::O,
    Mino::L,
    Mino::Z,
  ] {
    c.bench_function(&format!("collision map {}", mino.str()), |b| {
      b.iter(|| match mino {
        Mino::I => CollisionMap::usable::<{ Mino::I }>(black_box(&board)),
        Mino::J => CollisionMap::usable::<{ Mino::J }>(black_box(&board)),
        Mino::O => CollisionMap::usable::<{ Mino::O }>(black_box(&board)),
        Mino::T => CollisionMap::usable::<{ Mino::T }>(black_box(&board)),
        Mino::S => CollisionMap::usable::<{ Mino::S }>(black_box(&board)),
        Mino::Z => CollisionMap::usable::<{ Mino::Z }>(black_box(&board)),
        Mino::L => CollisionMap::usable::<{ Mino::L }>(black_box(&board)),
      });
    });
  }
}

criterion_group!(benches, collision_map);
criterion_main!(benches);
