use serde::{Deserialize, Serialize};
use triangle::types::game::Spin;

use crate::game::{BOARD_WIDTH, Game, HoleData};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weights {
  pub outer_height: f64,
  pub inner_height: f64,
  pub unevenness: f64,

  pub wells: [f64; BOARD_WIDTH],

  pub holes: HoleData<f64>,

  // spins
  pub clear: [[f64; 4]; 3],

  pub t_hole: f64,
  pub i_hole: f64,

  // waste mino type
  pub waste: [f64; 7],

  pub sent: f64,

  pub surge: f64,
  pub combo: f64,
  // TODO: kill chance/somehow boardwatching
}

pub struct MoveInfo {
	pub clear: (Spin, u8),
	pub sent: u16,
}

impl Weights {
  pub fn eval(self: &Self, state: &Game, move_info: &MoveInfo) -> f64 {
    let mut score = 0f64;

    let heights = &state.board.column_heights();
    let well = state.board.well(heights);

    score += match well {
      Some(idx) => self.wells[idx],
      None => 0.0,
    };

    let (outer, inner) = state.board.heights();
    score += self.outer_height * outer as f64;
    score += self.inner_height * inner as f64;

    score += self.unevenness * state.board.unevenness(heights, well) as f64;

    score += if move_info.clear.1 == 0 {
			0.0
		} else {
			self.clear[move_info.clear.0 as usize][move_info.clear.1 as usize - 1]
		};

    // TODO: hole types

    // TODO: waste mino type

    score += self.sent * move_info.sent as f64;

    score += self.surge
      * match state.b2b {
        4.. => state.b2b as f64,
        _ => 0.0,
      };
    score += self.combo * (state.combo + 1) as f64;

    score
  }
}

pub const WEIGHTS_HANDTUNED: Weights = Weights {
  outer_height: -5.0,
  inner_height: -10.0,
	unevenness: -3.0,

 wells: [-20.0, -30.0, -10.0, 0.0, 0.0, 0.0, 0.0, -10.0, -30.0, -20.0],
 
 clear: [
	 [-10.0, -10.0, -10.0, 50.0],
	 [20.0, 25.0, 30.0, 60.0],
	 [40.0, 80.0, 120.0, 180.0],
 ],

  sent: 20.0,

  surge: 8.0,
  combo: 3.0,

  holes: HoleData {
		holes: -10.0,
		depth: -5.0,
		accessible: 0.0,
		inaccessible: -20.0,
	},
	i_hole: 0.0,
	t_hole: 0.0,

	waste: [0.0; 7],
};
