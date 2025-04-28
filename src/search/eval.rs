use std::f32::consts::E;

use crate::game::{data::Spin, Game, BOARD_BUFFER, BOARD_HEIGHT};

pub struct Weights {
  pub height: i32,
  pub upper_half_height: i32,
  pub upper_quarter_height: i32,
  pub center_height: i32,

	pub clear_none: i32,
	pub clear_mini: i32,
	pub clear_normal: i32,

  pub sent: i32,

  pub b2b: i32,
  pub combo: i32,

  pub holes: i32,
  pub covered_holes: i32,

  pub unevenness: i32,
}

const HEIGHT: i32 = (BOARD_HEIGHT - BOARD_BUFFER) as i32;

const GENERAL_MULTIPLIER: i32 = 1000;
const GENERAL_MULTIPLIER_F32: f32 = GENERAL_MULTIPLIER as f32;

const WEIGHTS_HANDTUNED: Weights = Weights {
  height: -50,
  upper_half_height: -150,
  upper_quarter_height: -300,
  center_height: -100,

	clear_none: -5,
	clear_mini: 5,
	clear_normal: 10,

  sent: 0,

  b2b: 30,
  combo: 10,

  holes: -13,
  covered_holes: -30,

  unevenness: -3,
};

pub fn eval(state: &Game, sent: u16, clears: Vec<Spin>) -> i32 {
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

  score += (state.board.max_height()) * GENERAL_MULTIPLIER / HEIGHT * WEIGHTS_HANDTUNED.height;
  score += (state.board.upper_half_height()) * GENERAL_MULTIPLIER / HEIGHT
    * WEIGHTS_HANDTUNED.upper_half_height;
  score += (state.board.upper_quarter_height()) * GENERAL_MULTIPLIER / HEIGHT
    * WEIGHTS_HANDTUNED.upper_quarter_height;
  score +=
    (state.board.center_height()) * GENERAL_MULTIPLIER / HEIGHT * WEIGHTS_HANDTUNED.center_height;

	for c in clears {
		score += match c {
			Spin::None => WEIGHTS_HANDTUNED.clear_none,
			Spin::Mini => WEIGHTS_HANDTUNED.clear_mini,
			Spin::Normal => WEIGHTS_HANDTUNED.clear_normal,
		} * GENERAL_MULTIPLIER;
	}

  score += sent as i32 * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.sent;

  // add 1 so baseline is 0
  score += ((((state.b2b + 1) as f32 + E).ln() - 1.0) * GENERAL_MULTIPLIER_F32) as i32
    * WEIGHTS_HANDTUNED.b2b;
  score += (state.combo as i32 + 1) * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.combo;

  score += state.board.count_holes() * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.holes;
  score += state.board.covered_holes() * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.covered_holes;

  score += state.board.unevenness() * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.unevenness;

  score
}
