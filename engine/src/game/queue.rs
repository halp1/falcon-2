use std::collections::VecDeque;

use serde::Deserialize;
use triangle::engine::queue::Mino;

use super::rng::RNG;

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
pub struct Queue<const N: usize> {
  pub bag: Bag,
  pub rng: RNG,
  pub queue: VecDeque<Mino>,
}

impl<const N: usize> Queue<N> {
  pub fn new(bag: Bag, seed: u64, initial: Vec<Mino>) -> Self {
    let mut rng = RNG::new(seed);

    let mut queue: VecDeque<Mino> = VecDeque::with_capacity(N + 7);

    for m in initial.iter() {
      queue.push_back(*m);
    }

    while queue.len() < N {
      for mino in rng.shuffle(bag.get_cycle()) {
        queue.push_back(mino);
      }
    }

    Queue { bag, rng, queue }
  }

  pub fn shift(&mut self) -> Mino {
    let res = self
      .queue
      .pop_front()
      .unwrap_or_else(|| unreachable!("Queue is empty!"));

    while self.queue.len() < N {
      for mino in self.rng.shuffle(self.bag.get_cycle()) {
        self.queue.push_back(mino);
      }
    }

    res
  }

  pub fn as_array(&self) -> [Mino; N] {
    std::array::from_fn(|i| *self.queue.get(i).unwrap_or(&Mino::I))
  }
}
