use std::process::exit;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::{
  game::{
    data::Move, queue::{Bag, Queue}, Game, GameConfig, Garbage
  },
  keyfinder,
  search::{beam_search, eval::WEIGHTS_HANDTUNED},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Incoming {
  Start(Start),
  InsertGarbage(InsertGarbage),
  Step(Step),
}

#[derive(Deserialize)]
pub struct InsertGarbage {
  garbage: Vec<Garbage>,
}

#[derive(Deserialize)]
pub struct Start {
  pub config: GameConfig,
  pub seed: u64,
  pub bag: Bag,
}

#[derive(Deserialize)]
pub struct Step {
  garbage: Vec<Garbage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
  pub time: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Outgoing {
  Init { version: &'static str },
  Result { keys: Vec<Move>, stats: Stats },
  Crash { reason: &'static str },
}

pub async fn start_server() {
  let incoming = futures::stream::repeat_with(|| {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    serde_json::from_str::<Incoming>(&line).unwrap()
  });

  let outgoing = futures::sink::unfold((), |_, msg: Outgoing| {
    serde_json::to_writer(std::io::stdout(), &msg).unwrap();
    println!();
    async { Ok::<(), ()>(()) }
  });

  futures::pin_mut!(incoming);
  futures::pin_mut!(outgoing);

  outgoing
    .send(Outgoing::Init { version: "1.0.0-a" })
    .await
    .unwrap();

  let mut queue = Queue::new(Bag::Bag7, 0, 16, Vec::new());
  let mut game = Game::new(queue.shift(), queue.get_front_16());
  let mut config = Option::<GameConfig>::None;
  while let Some(msg) = incoming.next().await {
    match msg {
      Incoming::Start(start) => {
        queue = Queue::new(start.bag, start.seed, 16, Vec::new());
        config = Option::from(start.config);
        game = Game::new(queue.shift(), queue.get_front_16());
      }

      Incoming::InsertGarbage(garbage) => {
        for gb in garbage.garbage {
          game.board.insert_garbage(gb.amt, gb.col);
        }
				game.regen_collision_map();
      }

      Incoming::Step(cfg) => {
        if config.clone().is_none() {
          outgoing
            .send(Outgoing::Crash {
              reason: "Step requested without configuring (start message was never sent).",
            })
            .await
            .unwrap();
          exit(1);
        }
        game.garbage = cfg.garbage.into();

        // println!(
        //   "SEARCHING THROUGH: <{}> {:?}",
        //   game.piece.mino.str(),
        //   game.queue
        // );

        let start = std::time::Instant::now();
        let choice = beam_search(game.clone(), &(config.clone()).unwrap(), 10, &WEIGHTS_HANDTUNED);
        let elapsed = start.elapsed().as_secs_f64();

        if let Some(mv) = choice {
          let mut double_shift = false;
          if mv.0.3 {
            double_shift = game.hold.is_none();
            game.hold();
          }

          game.garbage.clear();

          let mut keys = keyfinder::get_keys(
            game.clone(),
            &config.clone().unwrap(),
            (mv.0.0, mv.0.1, mv.0.2, mv.0.4),
          );

          if mv.0.3 {
            keys.insert(0, Move::Hold);
          }

          for key in keys.iter() {
            key.run(&mut game, &config.clone().unwrap());
          }

          game.print();
					println!(
						"B2B: {}",
						game.b2b
					);

          game.hard_drop(&config.clone().unwrap());

          if double_shift {
            queue.shift();
          }
          queue.shift();
          game.queue = queue.get_front_16();
          game.queue_ptr = 0;

          outgoing
            .send(Outgoing::Result {
              keys: keys,
              stats: Stats { time: elapsed },
            })
            .await
            .unwrap();
        } else {
          game.hard_drop(&config.clone().unwrap());
          queue.shift();
          game.queue = queue.get_front_16();
          game.queue_ptr = 0;
          outgoing
            .send(Outgoing::Result {
              keys: vec![Move::HardDrop],
              stats: Stats { time: elapsed },
            })
            .await
            .unwrap();
        }
        game.regen_collision_map();
      }
    }
  }
}
