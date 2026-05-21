use rand::Rng;
use triangle::types::game::Key;

use engine::game::data::Move;

pub fn frames_till_next_piece(pieces: u32, pps: f64, time: f64, min_pps: f64, max_pps: f64) -> f64 {
  let res = ((pieces as f64 + 1.0) / pps) * 60.0 - time;
  let lower = 60.0 / max_pps;
  let upper = 60.0 / min_pps;
  lower.max(upper.min(res))
}

pub fn normal_random(mean: f64, stdev: f64) -> f64 {
  let mut rng = rand::rng();
  loop {
    let u: f64 = rng.random::<f64>() * 2.0 - 1.0;
    let v: f64 = rng.random::<f64>() * 2.0 - 1.0;
    let s = u * u + v * v;
    if s < 1.0 && s != 0.0 {
      let z = u * ((-2.0 * s.ln()) / s).sqrt();
      return z * stdev + mean;
    }
  }
}

pub fn move_to_key(m: Move) -> Key {
  match m {
    Move::Left => Key::MoveLeft,
    Move::Right => Key::MoveRight,
    Move::SoftDrop => Key::SoftDrop,
    Move::HardDrop => Key::HardDrop,
    Move::CW => Key::RotateCW,
    Move::CCW => Key::RotateCCW,
    Move::Flip => Key::Rotate180,
    Move::Hold => Key::Hold,
    Move::DasRight => Key::MoveRight,
    Move::DasLeft => Key::MoveLeft,
    Move::None => panic!("Move::None should not be in the move list"),
  }
}
