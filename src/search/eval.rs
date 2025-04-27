use std::f32::consts::E;

use crate::game::{BOARD_BUFFER, BOARD_HEIGHT, Game};

pub struct Weights {
  pub height: i32,
  pub center_height: i32,
  pub sent: i32,
  pub b2b: i32,
  pub holes: i32,
}

const HEIGHT: i32 = (BOARD_HEIGHT - BOARD_BUFFER) as i32;

const GENERAL_MULTIPLIER: i32 = 1000;
const GENERAL_MULTIPLIER_F32: f32 = GENERAL_MULTIPLIER as f32;

const WEIGHTS_HANDTUNED: Weights = Weights {
  height: -1,
  center_height: -10,
  sent: 0,
  b2b: 2,
  holes: -2,
};

pub fn eval(state: &Game, sent: u16) -> i32 {
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
  score +=
    (state.board.center_height()) * GENERAL_MULTIPLIER / HEIGHT * WEIGHTS_HANDTUNED.center_height;
  score += sent as i32 * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.sent;

  // add 1 so baseline is 0
  score += ((((state.b2b + 1) as f32 + E).ln() - 1.0) * GENERAL_MULTIPLIER_F32) as i32
    * WEIGHTS_HANDTUNED.b2b;
  score += state.board.count_holes() * GENERAL_MULTIPLIER * WEIGHTS_HANDTUNED.holes;

  score
}
