#![feature(stmt_expr_attributes)]

use num_format::{Locale, ToFormattedString};

use engine::game2::{
  board::Board,
  config::ConstConfig,
  data::{KickTable, Mino},
  map::CollisionMap,
  movegen::expand,
};

fn expand_mapped(mino: Mino, board: &Board) -> [CollisionMap; 3] {
  let h = board.real_height() as u8;
  #[rustfmt::skip]
	match mino {
		Mino::I => expand::<{ Mino::I }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::I }>(board), (4, h)),
		Mino::O => expand::<{ Mino::O }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::O }>(board), (4, h)),
		Mino::T => expand::<{ Mino::T }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::T }>(board), (4, h)),
		Mino::J => expand::<{ Mino::J }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::J }>(board), (4, h)),
		Mino::L => expand::<{ Mino::L }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::L }>(board), (4, h)),
		Mino::S => expand::<{ Mino::S }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::S }>(board), (4, h)),
		Mino::Z => expand::<{ Mino::Z }, { ConstConfig { kicktable: KickTable::SRS, enable_180: false }}>(&CollisionMap::usable::<{ Mino::Z }>(board), (4, h)),
	}
}

fn perft(board: &Board, queue: &[Mino], depth: usize) -> u64 {
  if depth == queue.len() - 1 {
    expand_mapped(queue[depth], board)[0].count_ones() as u64
  } else {
    let mut nodes = 0;
    let res = expand_mapped(queue[depth], board)[0];
    res.for_each_filled(queue[depth], |rot, x, y| {
      let mut b2 = board.clone();
      let blocks = queue[depth]
        .rot(rot)
        .map(|(bx, by)| ((x as i8 + bx) as u8, (y as i8 + by) as u8));

      if y >= 30 {
        board.print();
        res.data.iter().for_each(|b| b.print());
        panic!("invalid y value found: {}", y);
      }

      blocks
        .iter()
        .for_each(|&(x, y)| b2.set(x as usize, y as u8));

      b2.clear(0);

      nodes += perft(&b2, queue, depth + 1);
    });

    nodes
  }
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let queue = args[1].clone();

  let mino_queue = queue
    .chars()
    .map(|c| Mino::from_char(c))
    .collect::<Vec<_>>();

  let queue = mino_queue.as_slice();

  let start = std::time::Instant::now();
  let nodes = perft(&Board::new(), queue, 0);
  let duration = start.elapsed();

  println!(
    "Nodes: {}\nTime:  {:.3}s\nNPS:   {}",
    nodes,
    duration.as_secs_f64(),
    ((nodes as f64 / duration.as_secs_f64()) as u64).to_formatted_string(&Locale::en)
  )
}
