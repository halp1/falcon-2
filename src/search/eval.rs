use serde::{Deserialize, Serialize};

use crate::game::{BOARD_BUFFER, BOARD_HEIGHT, Game, data::Spin};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Weights {
  pub height: i32,
  pub upper_half_height: i32,
  pub upper_quarter_height: i32,
  pub center_height: i32,

  pub extra_wells: i32,

  pub clear_none: i32,
  pub clear_mini: i32,
  pub clear_normal: i32,

  pub sent: i32,

  pub b2b: i32,
  pub combo: i32,

  pub holes: i32,
  pub covered_holes: i32,
  pub overstacked_holes: i32,

  pub unevenness: i32,
}

impl Weights {
  pub fn mutate(&self, threshold: f32, change_factor: i32) -> Weights {
    let mut new_weights = self.clone();
    let get_multiplier = || {
      let v = rand::random::<f32>();
      if v < threshold {
        0
      } else if v < threshold + (1.0 - threshold) / 2.0 {
        1
      } else {
        -1
      }
    };

		new_weights.height = self.height + (change_factor * get_multiplier());
		new_weights.upper_half_height = self.upper_half_height + (change_factor * get_multiplier());
		new_weights.upper_quarter_height = self.upper_quarter_height + (change_factor * get_multiplier());
		new_weights.center_height = self.center_height + (change_factor * get_multiplier());
		new_weights.extra_wells = self.extra_wells + (change_factor * get_multiplier());
		new_weights.clear_none = self.clear_none + (change_factor * get_multiplier());
		new_weights.clear_mini = self.clear_mini + (change_factor * get_multiplier());
		new_weights.clear_normal = self.clear_normal + (change_factor * get_multiplier());
		new_weights.sent = self.sent + (change_factor * get_multiplier());
		new_weights.b2b = self.b2b + (change_factor * get_multiplier());
		new_weights.combo = self.combo + (change_factor * get_multiplier());
		new_weights.holes = self.holes + (change_factor * get_multiplier());
		new_weights.covered_holes = self.covered_holes + (change_factor * get_multiplier());
		new_weights.overstacked_holes = self.overstacked_holes + (change_factor * get_multiplier());
		new_weights.unevenness = self.unevenness + (change_factor * get_multiplier());

		new_weights
  }
}

const HEIGHT: i32 = (BOARD_HEIGHT - BOARD_BUFFER) as i32;
const HEIGHT_HALF: i32 = HEIGHT / 2;
const HEIGHT_QUARTER: i32 = HEIGHT / 4;

const GENERAL_MULTIPLIER: i32 = 1000;
const GENERAL_MULTIPLIER_F32: f32 = GENERAL_MULTIPLIER as f32;

pub const WEIGHTS_HANDTUNED: Weights = Weights {
  height: -50,
  upper_half_height: -150,
  upper_quarter_height: -300,
  center_height: -100,

  extra_wells: -100,

  clear_none: -70,
  clear_mini: 70,
  clear_normal: 140,

  sent: 0,

  b2b: 80,
  combo: 30,

  holes: -15,
  covered_holes: -60,
  overstacked_holes: -40,

  unevenness: -30,
};

pub const WEIGHTS_4W: Weights = Weights {
  height: -50,
  upper_half_height: -150,
  upper_quarter_height: -300,
  center_height: -100,

  extra_wells: -100,

  clear_none: 70,
  clear_mini: 140,
  clear_normal: 140,

  sent: 0,

  b2b: 0,
  combo: 10000,

  holes: 0,
  covered_holes: -60,
  overstacked_holes: -40,

  unevenness: -30,
};

pub fn eval(weights: &Weights, state: &Game, sent: u16, clears: Vec<Spin>) -> i32 {
  let mut score = 0;

  // state.print();
  // println!(
  //   "{} {} {} {} {}",
  //   state.board.max_height(),
  //   state.board.center_height(),
  //   sent,
  //   (state.b2b + 1),
  //   state.board.count_holes()
  // );

  score += (state.board.max_height()) * GENERAL_MULTIPLIER / HEIGHT * weights.height;
  score += (state.board.upper_half_height()) * GENERAL_MULTIPLIER / HEIGHT_HALF
    * weights.upper_half_height;
  score += (state.board.upper_quarter_height()) * GENERAL_MULTIPLIER / HEIGHT_QUARTER
    * weights.upper_quarter_height;
  score +=
    (state.board.center_height()) * GENERAL_MULTIPLIER / HEIGHT * weights.center_height;

  score += (state.board.wells()) * GENERAL_MULTIPLIER * weights.extra_wells;

  for c in clears {
    score += match c {
      Spin::None => weights.clear_none,
      Spin::Mini => weights.clear_mini,
      Spin::Normal => weights.clear_normal,
    } * GENERAL_MULTIPLIER;
  }

  score += sent as i32 * GENERAL_MULTIPLIER * weights.sent;

  // add 1 so baseline is 0
  score +=
    ((((state.b2b + 2) as f32).ln()) * GENERAL_MULTIPLIER_F32) as i32 * weights.b2b;
  score += (state.combo as i32 + 1) * GENERAL_MULTIPLIER * weights.combo;

  score += state.board.count_holes() * GENERAL_MULTIPLIER * weights.holes;
  score += state.board.covered_holes() * GENERAL_MULTIPLIER * weights.covered_holes;
  score +=
    state.board.overstacked_holes() * GENERAL_MULTIPLIER * weights.overstacked_holes;

  score += state.board.unevenness() * GENERAL_MULTIPLIER * weights.unevenness;

  score
}
