use engine::{
  game::{
    GameConfig,
    queue::{Bag, Queue},
  },
  search::{beam_search, eval::WEIGHTS_HANDTUNED},
};
use triangle::{
  engine::{queue::Mino, utils::KickTable},
  types::game::{ComboTable, SpinBonuses},
};

fn main() {
  let config = GameConfig {
    kicks: KickTable::SRSX,
    spins: SpinBonuses::Handheld,
    b2b_chaining: false,
    b2b_charging: true,
    b2b_charge_at: 0,
    b2b_charge_base: 0,
    pc_b2b: 1,
    pc_send: 5,
    combo_table: ComboTable::Multiplier,
    garbage_multiplier: 1.0,
    garbage_cap: 8,
    garbage_special_bonus: true,
    bag: Bag::Bag7,
  };

  let mut queue = Queue::new(Bag::Bag7, 0, vec![Mino::Z]);
  let game = engine::game::Game::new(queue.shift());

  let start_state = engine::game::StartState {
    queue: &queue.as_array(),
    garbage: &[],
  };

  let mut total = 0;

  for i in 0..105 {
    let start = std::time::Instant::now();
    beam_search::<7, 1000>(game.clone(), &config, &start_state, &WEIGHTS_HANDTUNED);
    if i > 5 {
      total += start.elapsed().as_millis();
    }
  }

  println!("avg: {}ms", total as f64 / 100.0);
}
