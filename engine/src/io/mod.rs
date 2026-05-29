use std::process::exit;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use triangle::engine::queue::Mino;

use crate::game::{
  Board, Game, GameConfig, Garbage, StartState,
  data::Move,
  queue::{Bag, Queue},
};
use crate::search::{beam_search, eval::WEIGHTS_HANDTUNED};

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
pub struct OpponentInfo {
  pub b2b: i16,
  pub combo: i16,
  pub board: Board,
  pub queue: Vec<Mino>,
  pub held: Option<Mino>,
}

#[derive(Deserialize)]
pub struct Step {
  garbage: Vec<Garbage>,
  opponent: OpponentInfo,
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

  let mut queue = Queue::<32>::new(Bag::Bag7, 0, Vec::new());
  let mut game = Game::new(queue.shift());
  let mut config = Option::<GameConfig>::None;
  while let Some(msg) = incoming.next().await {
    match msg {
      Incoming::Start(start) => {
        queue = Queue::<32>::new(start.bag, start.seed, Vec::new());
        config = Option::from(start.config);
        game = Game::new(queue.shift());
      }

      Incoming::InsertGarbage(garbage) => {
        for gb in garbage.garbage {
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
        let start_state = StartState {
          garbage: cfg.garbage.as_slice(),
          queue: &queue.as_array(),
        };
        game.garbage = (0, 0);

        // println!(
        //   "SEARCHING THROUGH: <{}> {:?}",
        //   game.piece.mino.str(),
        //   game.queue
        // );

        let mut opponent = Game::new(cfg.opponent.queue[0]);
        opponent.board = cfg.opponent.board;
        opponent.hold = cfg.opponent.held;
        opponent.b2b = cfg.opponent.b2b;
        opponent.combo = cfg.opponent.combo;

        let start = std::time::Instant::now();

        let choice = beam_search::<7, 1000>(
          game.clone(),
          &(config.clone()).unwrap(),
          &start_state,
          &WEIGHTS_HANDTUNED,
          WEIGHTS_HANDTUNED.eval_opponent(&opponent),
        );
        let elapsed = start.elapsed().as_secs_f64();

        if let Some(mv) = choice {
          let mut double_shift = false;
          if mv.0.hold {
            double_shift = game.hold.is_none();
            game.hold(&start_state);
          }

          let mut keys =
            crate::keyfinder::get_keys(game.clone(), &config.clone().unwrap(), mv.0.placement);

          let map = game.collision_map();

          for key in keys.iter() {
            key.run(&mut game, &config.clone().unwrap(), &map, &start_state);
          }

          if mv.0.hold {
            keys.insert(0, Move::Hold);
          }

          game.print();
          println!("B2B: {}", game.b2b);

          game.hard_drop(
            &config.clone().unwrap(),
            &map,
            &StartState {
              queue: &queue.as_array(),
              garbage: &[],
            },
            0,
          );

          if double_shift {
            queue.shift();
          }
          queue.shift();
          game.queue_ptr = 0;

          outgoing
            .send(Outgoing::Result {
              keys: keys,
              stats: Stats { time: elapsed },
            })
            .await
            .unwrap();
        } else {
          game.hard_drop(
            &config.clone().unwrap(),
            &game.collision_map(),
            &start_state,
            0,
          );
          queue.shift();
          game.queue_ptr = 0;
          outgoing
            .send(Outgoing::Result {
              keys: vec![Move::HardDrop],
              stats: Stats { time: elapsed },
            })
            .await
            .unwrap();
        }
      }
    }
  }
}
