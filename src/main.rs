mod game;
use game::data::Mino;

mod search;

fn main() {
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
  for _ in 0..7 {
    game.hard_drop();
    game.board.print();
  }
	game.board.insert_garbage(5, 2);
	game.board.print();
	game.rotate(3);
	game.das_left();
	// game.move_right();
	// game.move_right();
	game.hard_drop();
	game.board.print();
}
