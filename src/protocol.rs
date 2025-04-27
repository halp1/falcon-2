use std::process::exit;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::{
  game::{
    Game, GameConfig, Garbage,
    data::Move,
    queue::{Bag, Queue},
  },
  keyfinder,
  search::search,
};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Incoming {
  Start(Start),
  InsertGarbage(Vec<Garbage>),
  Step(Step),
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
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Outgoing {
  Init { version: &'static str },
  Result { keys: Vec<Move> },
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
      }

      Incoming::InsertGarbage(garbage) => {
        for gb in garbage {
          game.board.insert_garbage(gb.amt, gb.col);
        }
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

        let choice = search(game.clone(), &(config.clone()).unwrap(), 5);

        if let Some(mv) = choice {
          let mut double_shift = false;
          if mv.0.3 {
            double_shift = game.hold.is_none();
            game.hold();
          }
          game.piece.x = mv.0.0;
          game.piece.y = mv.0.1;
          game.piece.rot = mv.0.2;
          game.spin = mv.0.4;

          game.hard_drop(&config.clone().unwrap());

          if double_shift {
            queue.shift();
          }
          queue.shift();
          game.queue = queue.get_front_16();
          game.queue_ptr = 0;
          
          let mut keys = keyfinder::get_keys(
            game.clone(),
            &config.clone().unwrap(),
            (mv.0.0, mv.0.1, mv.0.2, mv.0.4),
          );

          if mv.0.3 {
            keys.insert(0, Move::Hold);
          }

          outgoing
            .send(Outgoing::Result {
              keys: keys,
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
            })
            .await
            .unwrap();
        }
        game.regen_collision_map();
      }
    }
  }
}
