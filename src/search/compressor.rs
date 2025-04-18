use crate::game::{Game, data::Move};

pub fn compress_key(state: &Game, last_move: Move) -> u16 {
  let mut res = 0u16;

  res |= state.piece.x as u16 & 0b_1111;
  res |= (state.piece.y as u16 & 0b_111111) << 4;
  res |= (state.piece.rot as u16 & 0b11) << 10;
  res |= (last_move as u16 & 0b1111) << 12;
  res
}

pub fn decompress_key(state: u16) -> (u8, u8, u8, Move) {
  let x = (state & 0b_1111) as u8;
  let y = ((state >> 4) & 0b_111111) as u8;
  let rot = ((state >> 10) & 0b11) as u8;

  (
    x,
    y,
    rot,
    match (state >> 12) & 0b1111 {
      0 => Move::None,
      1 => Move::Left,
      2 => Move::Right,
      3 => Move::SoftDrop,
      4 => Move::CCW,
      5 => Move::CW,
      6 => Move::Flip,
      _ => unreachable!(),
    },
  )
}

pub fn compress_move(state: &Game, hold: bool) -> u16 {
  let mut res = 0u16;

  res |= state.piece.x as u16 & 0b_1111;
  res |= (state.piece.y as u16 & 0b_111111) << 4;
  res |= (state.piece.rot as u16 & 0b11) << 10;
  res |= (if hold { 1 } else { 0 } as u16 & 0b1) << 12;
  res
}

pub fn decompress_move(state: u16) -> (u8, u8, u8, bool) {
  let x = (state & 0b_1111) as u8;
  let y = ((state >> 4) & 0b_111111) as u8;
  let rot = ((state >> 10) & 0b11) as u8;
  let hold = ((state >> 12) & 0b1) != 0;

  (x, y, rot, hold)
}

pub fn convert_compressed(mut key: u16, hold: bool) -> u16 {
  key = key & 0b_0000_1111_1111_1111;

  if hold {
    key |= 0b_0001_0000_0000_0000;
  }

  key
}
