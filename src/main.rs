mod game;
pub mod keyfinder;

use game::{
  data::{Mino, Spin, Spins},
  queue::{Bag, Queue},
};
use keyfinder::get_keys;
use std::time::Instant;

mod search;

mod protocol;

fn main() {
  // futures::executor::block_on(protocol::start_server());

  let config = &game::GameConfig {
    spins: Spins::MiniPlus,
    b2b_chaining: true,
    b2b_charging: false,
    b2b_charge_at: 0,
    b2b_charge_base: 0,
    pc_b2b: 0,
    combo_table: game::data::ComboTable::Multiplier,
    garbage_multiplier: 1.0,
    garbage_special_bonus: false,
  };

  let mut queue = Queue::new(Bag::Bag7, 0, 16, Vec::from([]));

  let mut game = game::Game::new(queue.shift(), queue.get_front_16());
  game.board.cols = [
    0b11u64, 0b11u64, 0b11u64, 0b01u64, 0b00u64, 0b10u64, 0b11u64, 0b11u64, 0b11u64, 0b11u64,
  ];
  game.regen_collision_map();

  // for _ in 0..7 {
  //   game.hard_drop(config);
  //   game.board.print();
  // }
  // game.board.insert_garbage(5, 2);
  // game.collision_map = game.board.collision_map(&game.piece);
  // game.board.print();
  // game.das_left();
  // game.soft_drop();
  // game.rotate(3);
  // game.das_left();
  // print_board(Vec::from(game.collision_map.states[game.piece.rot as usize]), 0);
  // game.hard_drop(config);
  // game.board.print();

  // -- EXPANSION TEST --

  // let mut avg_time = 0f32;
  // let iters = 100000;

  // let passed = &mut [0u64; 2048];
  // let res = &mut [(0, 0, 0, Spin::None); 512];

  // for i in 0..iters + 5 {
  //   let mut g = game.clone();
  //   let start = Instant::now();
  //   let r = search::expand(&mut g, config, passed, res);
  //   let duration = start.elapsed();
  //   if i == iters + 5 - 1 {
  //     println!("Total positions found for {}: {}", game.piece.mino.str(), r);

  //     for j in 0..r {
  //       let mut tester = game.clone();
  //       let search_game = game.clone();
  //       let (x, y, rot, spin) = res[j];
  //       tester.piece.x = x;
  //       tester.piece.y = y;
  //       tester.piece.rot = rot;
  //       tester.spin = spin;
  //       println!("{} {} {} {}", x, y, rot, spin.str());
  //       tester.print();
  //       tester.hard_drop(config);
  //       search_game.print();
  //       let key_start = Instant::now();
  //       let keys = get_keys(search_game, config, (x, y, rot, spin));
  //       println!(
  //         "{:?} in {} us",
  //         keys,
  //         key_start.elapsed().as_secs_f32() * 1_000_000.0
  //       );

  //       println!("------------------------");
  //     }
  //   }
  //   if i > 5 {
  //     avg_time += duration.as_secs_f32();
  //   }
  // }

  // avg_time /= iters as f32;
  // println!("Average search time: {:?}us", avg_time * 1_000_000.0);

  // -- SEARCH TEST --

  // println!(
  //   "SEARCHING THROUGH: <{}> {:?}",
  //   game.piece.mino.str(),
  //   game.queue
  // );

  // let mut avg_time = 0f32;
  // let iters = 1000000;
  // game.print();
  // game.print();

  // for i in 0..iters + 5 {
  //   let g = game.clone();
  //   let start = Instant::now();
  //   let res = search::search(g, config, 1).unwrap();
  //   let duration = start.elapsed();
  //   if i == iters + 5 - 1 {
  //     let g = res.1.clone();
  //     g.board.print();
  //     println!("Stats:");
  //     println!("B2B: {}", g.b2b);
  // 		println!("TARGET: {} {} {} {}", res.0.0, res.0.1, res.0.2, res.0.3);
  // 		let mut g = game.clone();
  // 		g.piece.x = res.0.0;
  // 		g.piece.y = res.0.1;
  // 		g.piece.rot = res.0.2;

  // 		g.print();
  //   }
  //   if i > 5 {
  //     avg_time += duration.as_secs_f32();
  //   }
  // }
  // avg_time /= iters as f32;
  // println!("Average search time: {:?}ms", avg_time * 1000.0);

  // -- PLAY TEST --

  loop {
  	println!("SEARCHING THROUGH: <{}> {:?}", game.piece.mino.str(), game.queue);
  	let res = search::search(game.clone(), config, 4).unwrap();

  	if res.0.3 {
  		game.hold();
  	}

  	game.piece.x = res.0.0;
  	game.piece.y = res.0.1;
  	game.piece.rot = res.0.2;
  	game.spin = res.0.4;

  	println!("PROJECTION ({} b2b):", res.1.b2b);
  	res.1.board.print();
  	println!("{} {} {} {}", game.piece.mino.str(), res.0.0, res.0.1, res.0.2);

  	game.print();
  	game.hard_drop(config);
  	game.regen_collision_map();

  	queue.shift();
  	game.queue_ptr = 0;
  	game.queue = queue.get_front_16();

  	println!("CURRENT ({} b2b):", game.b2b);

  	if game.topped_out() {
  		break;
  	}
  }

  println!("Game over, topped out");
}
