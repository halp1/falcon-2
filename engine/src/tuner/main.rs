pub mod cmaes;
pub mod sim;
pub mod spsa;

use engine::{
  game::{GameConfig, queue::Bag},
  search::eval::Weights,
};
use triangle::{
  engine::utils::KickTable,
  types::game::{ComboTable, SpinBonuses},
};

fn load_checkpoint() -> Option<Weights> {
  let json = std::fs::read_to_string("tuning/weights_checkpoint.json").ok()?;
  serde_json::from_str(&json).ok()
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let continue_iter = args
    .windows(2)
    .find(|w| w[0] == "--continue")
    .and_then(|w| w[1].parse::<usize>().ok());

  let (start_iter, initial) = if let Some(x) = continue_iter {
    let weights = load_checkpoint()
      .expect("--continue passed but tuning/weights_checkpoint.json could not be loaded");
    println!("Resuming from iteration {x} with checkpoint weights");
    (x, Some(weights))
  } else {
    (0, None)
  };

  let config = GameConfig {
    kicks: KickTable::SRSPlus,
    spins: SpinBonuses::AllMiniPlus,
    b2b_chaining: false,
    b2b_charging: true,
    b2b_charge_at: 4,
    b2b_charge_base: 3,
    pc_b2b: 1,
    pc_send: 5,
    combo_table: ComboTable::Multiplier,
    garbage_multiplier: 1.0,
    garbage_cap: 8,
    garbage_special_bonus: true,
    bag: Bag::Bag7,
  };

  // cmaes::tune::<6, 60>(config, 1000, 4, 8, 1000, start_iter, initial);
  spsa::tune::<6, 60>(config, 1000, 32, 500, 500.0, 10.0, start_iter, initial);
}
