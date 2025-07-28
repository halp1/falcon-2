use std::collections::VecDeque;

use serde::Deserialize;

use super::{data::Mino, rng::RNG};

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum Bag {
	#[serde(rename = "7-bag")]
  Bag7,
}

impl Bag {
  pub fn get_cycle(&self) -> Vec<Mino> {
    match self {
      Bag::Bag7 => Vec::from([
        Mino::Z,
        Mino::L,
        Mino::O,
        Mino::S,
        Mino::I,
        Mino::J,
        Mino::T,
      ]),
    }
  }
}

#[derive(Clone)]
pub struct Queue {
  pub bag: Bag,
  pub rng: RNG,
  pub min_size: usize,
  pub queue: VecDeque<Mino>,
}

impl Queue {
  pub fn new(bag: Bag, seed: u64, min_size: usize, initial: Vec<Mino>) -> Self {
    assert!(min_size >= 32, "Bag min size must be at least 32");
    let mut rng = RNG::new(seed);

    let mut queue: VecDeque<Mino> = VecDeque::with_capacity(min_size + 7);

		for m in initial.iter() {
			queue.push_back(*m);
		}

    while queue.len() < min_size {
      for mino in rng.shuffle(bag.get_cycle()) {
        queue.push_back(mino);
      }
    }

    Queue {
      bag,
      rng,
      min_size,
      queue,
    }
  }

  pub fn shift(&mut self) -> Mino {
    let res = self.queue.pop_front().unwrap_or_else(|| unreachable!("Queue is empty!"));

    while self.queue.len() < self.min_size {
      for mino in self.rng.shuffle(self.bag.get_cycle()) {
        self.queue.push_back(mino);
      }
    }

    res
  }

  pub fn get_front_32(&self) -> [Mino; 32] {
    let mut res = [Mino::I; 32];

    for i in 0usize..32 {
      res[i] = *self.queue.get(i).unwrap_or(&Mino::I);
    }

    res
  }
}
