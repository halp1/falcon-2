mod game;
use game::data::Mino;
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

  let mut game = game::Game::new(vec![
    Mino::L,
    Mino::S,
    Mino::T,
    Mino::I,
    Mino::J,
    Mino::Z,
    Mino::O,
    Mino::I,
    Mino::O,
  ]);
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

	let start = Instant::now();
	let res = search::search(game, config, 3);
	let duration = start.elapsed();
	println!("Search completed in: {:?}", duration);
}
