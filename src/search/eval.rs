use std::f32::consts::E;

use crate::game::{Game, BOARD_BUFFER, BOARD_HEIGHT};

pub struct Weights {
	pub height: f32,
	pub center_height: f32,
	pub sent: f32,
	pub b2b: f32,
}

const HEIGHT: f32 = (BOARD_HEIGHT - BOARD_BUFFER) as f32;

const WEIGHTS_HANDTUNED: Weights = Weights {
	height: -0.2,
	center_height: -0.5,
	sent: 0.2,
	b2b: 0.5,
};

pub fn eval(state: &Game, sent: u16) -> f32 {
	let mut score = 0.0;

	score += (state.board.max_height() as f32) / HEIGHT * WEIGHTS_HANDTUNED.height;
	score += (state.board.center_height() as f32) / HEIGHT * WEIGHTS_HANDTUNED.center_height;
	score += sent as f32 * WEIGHTS_HANDTUNED.sent;
	score += (((state.b2b * 2 + 1) as f32 + E) * WEIGHTS_HANDTUNED.b2b).ln().max(0.0);

	score
}