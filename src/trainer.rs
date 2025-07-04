use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::{
  game::{
    Game, GameConfig, Garbage,
    queue::{Bag, Queue},
  },
  search::{self, eval::Weights},
};

#[derive(Clone)]
struct Player {
  weights: Weights,
  performance: u32,
}
fn play_match(config: &GameConfig, w1: &Weights, w2: &Weights) -> u8 {
  let mut q1 = Queue::new(Bag::Bag7, rand::random::<u64>(), 16, Vec::new());
  let mut q2 = q1.clone();
  let mut g1 = Game::new(q1.shift(), q1.get_front_16());
  let mut g2 = Game::new(q2.shift(), q2.get_front_16());

  loop {
    // player 1’s move
    let mv1 = match search::beam_search(g1.clone(), config, 7, w1) {
      None => return 2, // p1 loses
      Some(play) => play.0,
    };
    if mv1.3 {
			let double_shift = g1.hold.is_none();
      g1.hold();
			if double_shift {
				q1.shift();
			}
			g1.regen_collision_map();
    }
    g1.piece.x = mv1.0;
    g1.piece.y = mv1.1;
    g1.piece.rot = mv1.2;
    g1.spin = mv1.4;
    let send1 = g1.hard_drop(config).0;

    // player 2’s move
    let mv2 = match search::beam_search(g2.clone(), config, 7, w2) {
      None => return 1, // p2 loses
      Some(play) => play.0,
    };
    if mv2.3 {
			let double_shift = g2.hold.is_none();
      g2.hold();
			if double_shift {
				q2.shift();
			}
			g2.regen_collision_map();
    }
    g2.piece.x = mv2.0;
    g2.piece.y = mv2.1;
    g2.piece.rot = mv2.2;
    g2.spin = mv2.4;
    let send2 = g2.hard_drop(config).0;

    // topped‐out checks
    if g1.topped_out() {
      return 2;
    }
    if g2.topped_out() {
      return 1;
    }

    // garbage exchange
    if send1 > send2 {
      g2.garbage.push_back(Garbage {
        amt: (send1 - send2) as u8,
        col: rand::random::<u8>() % 10,
        time: 1,
      });
    } else if send2 > send1 {
      g1.garbage.push_back(Garbage {
        amt: (send2 - send1) as u8,
        col: rand::random::<u8>() % 10,
        time: 1,
      });
    }

		g1.regen_collision_map();
		g2.regen_collision_map();

		q1.shift();
		q2.shift();
		g1.queue_ptr = 0;
		g2.queue_ptr = 0;
		g1.queue = q1.get_front_16();
		g2.queue = q2.get_front_16();

		// check for game over
		if g1.topped_out() {
			return 2;
		}
		if g2.topped_out() {
			return 1;
		}
  }
}

pub fn train(config: &GameConfig, initial: Weights, num_players: usize, epochs: u32) -> Weights {
  // initialize players
  let mut players = vec![
    Player {
      weights: initial.clone(),
      performance: 0
    };
    num_players
  ].iter().map(|p| {
		let mut pl = p.clone();
		pl.weights = pl.weights.mutate(0.5, 20);
		pl
	}).collect::<Vec<Player>>();

  for epoch in 0..epochs {
    // snapshot weights so all matches see the same epoch’s weights.
    let epoch_weights: Vec<_> = players.iter().map(|p| p.weights.clone()).collect();

    // prepare a thread‐safe counter for each player
    let perf_counters = (0..num_players)
      .map(|_| AtomicU32::new(0))
      .collect::<Vec<_>>();

    // build all (i,j) pairs with i<j
    let pairs: Vec<(usize, usize)> = (0..num_players)
      .flat_map(|i| ((i + 1)..num_players).map(move |j| (i, j)))
      .collect();

    // run all matches in parallel
    pairs.par_iter().for_each(|&(i, j)| {
      let victor = play_match(config, &epoch_weights[i], &epoch_weights[j]);
			println!("{} vs {}: {}", i, j, victor);
      if victor == 1 {
        perf_counters[i].fetch_add(1, Ordering::Relaxed);
      } else {
        perf_counters[j].fetch_add(1, Ordering::Relaxed);
      }
    });

    // write back the performances
    for i in 0..num_players {
      players[i].performance = perf_counters[i].load(Ordering::Relaxed);
    }

    // if last epoch, skip breeding/mutation
    if epoch == epochs - 1 {
      break;
    }

    // select top 25%
    let mut sorted = players.clone();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.performance));
    let top_quart = &sorted[..(num_players / 4)];

    // redistribute & mutate in parallel
    players.par_iter_mut().enumerate().for_each(|(i, player)| {
      player.performance = 0;
      let parent = &top_quart[i % top_quart.len()].weights;
      player.weights = parent.clone().mutate(0.1, 3);
    });
  }

  // pick the best performer
  players
    .into_iter()
    .max_by_key(|p| p.performance)
    .unwrap()
    .weights
}
