pub mod cmaes;
pub mod sim;
pub mod spsa;

use engine::game::{GameConfig, queue::Bag};
use triangle::{
  engine::utils::KickTable,
  types::game::{ComboTable, SpinBonuses},
};

fn main() {
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

  cmaes::tune::<6, 60>(config, 1000, 4, 8, 1000);
}
