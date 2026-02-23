mod game;
pub mod keyfinder;
pub mod trainer;

use game::{
  data::{Spin, Spins},
  queue::{Bag, Queue},
};
use keyfinder::get_keys;
use std::time::Instant;

use rand;

mod search;

mod protocol;

fn main() {
  // futures::executor::block_on(protocol::start_server());

  tests::test_expansion();
}

pub mod tests {
  use super::*;
  use crate::{
    game::{
      BOARD_HEIGHT, BOARD_WIDTH, Game,
      data::{KickTable, Mino},
      print_board,
    },
    search::eval::{WEIGHTS_4W, WEIGHTS_HANDTUNED},
  };

  pub fn init() -> (game::GameConfig, Queue, Game) {
    let config = game::GameConfig {
      kicks: KickTable::SRSX,
      spins: Spins::Handheld,
      b2b_chaining: false,
      b2b_charging: true,
      b2b_charge_at: 0,
      b2b_charge_base: 0,
      pc_b2b: 1,
      pc_send: 5,
      combo_table: game::data::ComboTable::Multiplier,
      garbage_multiplier: 1.0,
      garbage_special_bonus: true,
    };

    let mut queue = Queue::new(Bag::Bag7, rand::random::<u64>(), 32, vec![Mino::Z]);
    // let mut queue = Queue::new(Bag::Bag7, 2, 32, Vec::from([]));

    let game = game::Game::new(queue.shift(), queue.get_front_32());

    (config, queue, game)
  }

  pub fn test_spins() {
    let (config, _, _) = init();

    let mut queue = Queue::new(Bag::Bag7, 0, 32, Vec::from([Mino::T]));

    let mut game = game::Game::new(queue.shift(), queue.get_front_32());
    println!("{}", game.piece.y);

    let points = [(0, 0), (1, 0), (2, 0)];

    for point in points.iter() {
      game.board.set(point.0, point.1);
    }

    game.regen_collision_map();

    println!("Initial board:");
    game.print();

    game.move_right();
    game.soft_drop();
    game.rotate(3, &config);

    println!("{:?}", game.spin);

    game.print();
  }

  pub fn test_game() {
    let (config, _, _) = init();

    let mut queue = Queue::new(Bag::Bag7, 0, 32, Vec::from([Mino::T]));

    let mut game = game::Game::new(queue.shift(), queue.get_front_32());

    let points = [
      (0, 0),
      (1, 0),
      (2, 0),
      (0, 1),
      (0, 2),
      (0, 3),
      (1, 3),
      (1, 4),
    ];

    for point in points.iter() {
      game.board.set(point.0, point.1);
    }

    game.regen_collision_map();

    println!("Initial board:");
    game.print();

    let keys = get_keys(game.clone(), &config, (4, 2, 3, Spin::Mini));
    println!("Keys: {:?}", keys);

    for key in keys.iter() {
      key.run(&mut game, &config);
      println!("After key: {:?}", key);
      game.print();
    }
  }

  pub fn test_expansion() {
    let (config, _, game) = init();
    let config = &config;

    let mut avg_time = 0f32;
    let iters = 100_000_00;

    let passed = &mut [0u64; 2048];
    // let passed = &mut [0u64; BOARD_WIDTH * BOARD_HEIGHT];
    let res = &mut [(0, 0, 0, Spin::None); 512];

    for i in 0..iters + 5 {
      let mut g = game.clone();
      let start = Instant::now();
      let r = search::expand(&mut g, config, passed, res);
      let duration = start.elapsed();
      if i == iters + 5 - 1 {
        println!(
          "Total positions found for {}: {}",
          game.piece.mino.str(),
          r.0
        );

        for j in 0..r.0 {
          let mut tester = game.clone();
          let search_game = game.clone();
          let (x, y, rot, spin) = res[j];
          tester.piece.x = x;
          tester.piece.y = y;
          tester.piece.rot = rot;
          tester.spin = spin;
          println!("{} {} {} {}", x, y, rot, spin.str());
          tester.print();
          tester.hard_drop(config);
          let key_start = Instant::now();
          let keys = get_keys(search_game, config, (x, y, rot, spin));
          println!(
            "{:?} in {} us",
            keys,
            key_start.elapsed().as_secs_f32() * 1_000_000.0
          );

          println!("------------------------");
        }
      }
      if i > 5 {
        avg_time += duration.as_secs_f32();
      }
    }

    println!("NPS: {:?}", (iters as f32) / avg_time);
    avg_time /= iters as f32;
    println!("Average search time: {:?}us", avg_time * 1_000_000.0);
  }

  pub fn bench_expansion() {
    let (config, _, mut game) = init();
    let config = &config;

    let iters = 10000000;

    let mut nodes = 0;

    // let passed = &mut [0u64; 2048];
    let passed = &mut [0u64; BOARD_WIDTH * BOARD_HEIGHT];
    let res = &mut [(0, 0, 0, Spin::None); 512];

    let start = Instant::now();

    let (x, y, rot, spin) = (game.piece.x, game.piece.y, game.piece.rot, game.spin);

    for _ in 0..iters {
      game.piece.x = x;
      game.piece.y = y;
      game.piece.rot = rot;
      game.spin = spin;
      let r = search::expand_floodfill(&mut game, config, passed, res);
      nodes += r.1;
    }

    let duration = start.elapsed();
    println!("NPS: {:?}", nodes as f32 / duration.as_secs_f32());
  }

  pub fn test_search() {
    let (config, _, game) = init();
    let config = &config;

    println!(
      "SEARCHING THROUGH: <{}> {:?}",
      game.piece.mino.str(),
      game.queue
    );

    let mut avg_time = 0f32;
    let iters = 10_000;
    game.print();
    game.print();

    for i in 0..iters + 5 {
      let g = game.clone();
      let start = Instant::now();
      let res = search::beam_search(g, config, 10, &WEIGHTS_HANDTUNED).unwrap();
      let duration = start.elapsed();
      if i == iters + 5 - 1 {
        let g = res.1.clone();
        g.board.print();
        println!("Stats:");
        println!("B2B: {}", g.b2b);
        println!("TARGET: {} {} {} {}", res.0.0, res.0.1, res.0.2, res.0.3);
        let mut g = game.clone();
        g.piece.x = res.0.0;
        g.piece.y = res.0.1;
        g.piece.rot = res.0.2;

        g.print();
      }
      if i > 5 {
        avg_time += duration.as_secs_f32();
      }
    }
    avg_time /= iters as f32;
    println!("Average search time: {:?}ms", avg_time * 1000.0);
  }

  pub fn test_play() {
    let (config, mut queue, mut game) = init();
    let config = &config;

    let mut count = 0;
    let mut attack = 0;

    let mut max_combo = -1;

    loop {
      count += 1;

      println!(
        "SEARCHING THROUGH: <{}> {:?}",
        game.piece.mino.str(),
        game.queue
      );
      let start = Instant::now();
      let res = search::beam_search(game.clone(), config, 10, &WEIGHTS_HANDTUNED);
      let elapsed = start.elapsed();
      if res.is_none() {
        println!("NO SOLUTION FOUND");
        break;
      }

      let res = res.unwrap();

      if res.0.3 {
        game.hold();
        game.regen_collision_map();
        queue.shift();
      }

      let (x, y, rot, spin) = (res.0.0, res.0.1, res.0.2, res.0.4);

      let keys = get_keys(game.clone(), config, (x, y, rot, spin));
      for key in keys.iter() {
        key.run(&mut game, config);
      }

      println!("PROJECTION ({} combo):", res.1.combo);
      res.1.board.print();
      println!(
        "{} {} {} {}",
        game.piece.mino.str(),
        res.0.0,
        res.0.1,
        res.0.2
      );

      game.print();
      println!("KEYS: {:?}", keys);

      attack += game.hard_drop(config).0;
      game.regen_collision_map();

      // if count % 15 == 0 {
      //   // clean
      //   if rand::random_bool(1.0) {
      //     game.garbage.push_back(Garbage {
      //       amt: 4,
      //       col: rand::random::<u8>() % 10,
      //       time: 0,
      //     });
      //   } else {
      //     // cheese
      //     for _ in 0..2 {
      //       game.garbage.push_back(Garbage {
      //         amt: 1,
      //         col: rand::random::<u8>() % 10,
      //         time: 0,
      //       });
      //     }
      //   }
      // }

      queue.shift();
      game.queue_ptr = 0;
      game.queue = queue.get_front_32();

      println!("B2B: {}", game.b2b);
      println!("COMBO: {}", game.combo);
      println!("PIECE #: {}", count);
      println!("TIME: {}ms", elapsed.as_secs_f32() * 1000.0);
      println!("APP: {:.2}", attack as f32 / count as f32);

      max_combo = std::cmp::max(max_combo, game.combo);
      println!("MAX COMBO: {}", max_combo);

      if game.topped_out() {
        break;
      }
    }

    println!(
      "Game over, topped out ({} @seed {}):",
      game.topped_out(),
      queue.rng.seed
    );
    game.print();
  }

  pub fn train() {
    let (config, _, _) = init();

    let res = trainer::train(&config, WEIGHTS_HANDTUNED, 10, 10);

    println!(
      "Trained weights: {}",
      serde_json::to_string_pretty(&res).unwrap()
    );
  }
}
