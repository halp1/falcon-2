const MODULUS: u64 = 2147483647;
const MULTIPLIER: u64 = 16807;
const MAX_FLOAT: u64 = 2147483646;

#[derive(Clone, Copy)]
pub struct RNG {
  pub seed: u64,
  pub index: usize,
}

impl RNG {
  pub fn new(seed: u64) -> Self {
    let mut rng = RNG {
      seed: seed % MODULUS,
      index: 0,
    };

    if rng.seed <= 0 {
      rng.seed += MAX_FLOAT;
    }

    rng
  }
  pub fn next(&mut self) -> u64 {
    self.index += 1;
    self.seed = (MULTIPLIER * self.seed) % MODULUS;
    self.seed
  }

  pub fn next_float(&mut self) -> f64 {
    (self.next() - 1) as f64 / MAX_FLOAT as f64
  }

  pub fn shuffle<T: Clone>(&mut self, mut array: Vec<T>) -> Vec<T> {
    if array.is_empty() {
      return array;
    }

    for i in (1..array.len()).rev() {
      let r = (self.next_float() * (i + 1) as f64) as usize;
      array.swap(i, r);
    }

    array
  }

  pub fn set_seed(&mut self, value: u64) {
    self.seed = value % MODULUS;

    if self.seed <= 0 {
      self.seed += MAX_FLOAT;
    }
  }
}
