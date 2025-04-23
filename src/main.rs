mod game;
use game::{data::Mino, queue::{Bag, Queue}};
use std::time::Instant;

mod search;


fn main() {
  let config = &game::GameConfig {
    b2b_chaining: true,
    b2b_charging: false,
    b2b_charge_at: 0,
    b2b_charge_base: 0,
    pc_b2b: 0,
    combo_table: game::data::ComboTable::Multiplier,
    garbage_multiplier: 1.0,
    garbage_special_bonus: false,
  };

	let mut queue = Queue::new(Bag::Bag7, 0, 16);

  let mut game = game::Game::new(queue.shift(), queue.get_front_16());
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

  // expansion test
  // let start = Instant::now();
  // let res = search::expand(game.clone(), config);
  // let duration = start.elapsed();

  // for (game, _) in res.iter() {
  //   game.board.print();
  // }

  // println!("Total games found: {}", res.len());
  // println!("Search completed in: {:?}", duration);

  // let mut avg_time = 0f32;
  // let iters = 10;

  // for i in 0..iters + 5 {
  //   let g = game.clone();
  //   let start = Instant::now();
  //   let res = search::search(g, config, 4);
  //   let duration = start.elapsed();
  //   if i == iters + 5 - 1 {
  //     res.unwrap().1.board.print();
  //   }
  //   if i > 5 {
  //     avg_time += duration.as_secs_f32();
  //   }
  // }
  // avg_time /= iters as f32;
  // println!("Average search time: {:?}ms", avg_time * 1000.0);

	loop {
		let res = search::search(game.clone(), config, 4).unwrap();

		game.piece.x = res.0.0;
		game.piece.y = res.0.1;
		game.piece.rot = res.0.2;

		res.1.board.print();
		println!("{} {} {} {}", game.piece.mino.str(), res.0.0, res.0.1, res.0.2);

		game.hard_drop(config);

		queue.shift();
		game.queue_ptr = 0;
		game.queue = queue.get_front_16();

		game.board.print();
	}
}
