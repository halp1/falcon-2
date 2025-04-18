use super::data::{ComboTable, Spin};

pub fn damage_calc(
  lines: u8,
  spin: Spin,
  b2b: i16,
  combo: i16,
  combo_table: ComboTable,
  b2b_chaining: bool,
) -> f32 {
  assert!(lines <= 4, "Lines must be between 0 and 4");
  // Skip formatting for next 7 lines
  #[rustfmt::skip]
	let mut damage: f32 = match lines {
		0 => 0.0,
		1 => if spin == Spin::Normal { 2.0 } else { 0.0 },
		2 => if spin == Spin::Normal { 4.0 } else { 1.0 },
		3 => if spin == Spin::Normal { 6.0 } else { 2.0 },
		4 => if spin != Spin::None { 10.0 } else { 4.0 },
		_ => panic!("Invalid number of lines: {}", lines),
	};

  damage += if lines > 0 && b2b > 0 {
    if b2b_chaining {
      (1.0 + (b2b as f32 * 0.8).ln_1p()).floor()
        + if b2b == 1 {
          0.0
        } else {
          (1.0 + (b2b as f32 * 0.8).ln_1p().fract()) / 3.0
        }
    } else {
      1.0
    }
  } else {
    0.0
  };

  damage = if combo > 0 {
    if combo_table == ComboTable::Multiplier {
      let g1 = damage * (1.0 + 0.25 * combo as f32);
      if combo > 1 {
        (combo as f32 * 1.25).ln_1p().max(g1)
      } else {
        g1
      }
    } else {
      let t = combo_table.get();
      damage + t[(combo - 1).clamp(0, t.len() as i16 - 1) as usize] as f32
    }
  } else {
    damage
  };

  damage
}
