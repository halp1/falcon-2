use core::panic;
use std::{env, sync::Arc};

use triangle::{
  Client, ClientOptions, Credentials,
  classes::{
    client::RibbonOptions,
    ribbon::{self},
  },
  engine::{queue::bag::BagType, utils::KickTable},
  types::{
    events::recv,
    game::{Key, tick},
    room::Bracket,
  },
};

use crate::engine::{Falcon, game::data::Move};

struct FrameCounter(f64);
impl FrameCounter {
  pub fn new(v: u64) -> Self {
    Self(v as f64)
  }

  pub fn add(&mut self, delta: f64) {
    self.0 = ((self.0 + delta) * 10.0).round() / 10.0;
  }

  pub fn frame(&self) -> u64 {
    self.0.floor() as u64
  }

  pub fn subframe(&self) -> f64 {
    ((self.0 - self.0.floor()) * 10.0).round() / 10.0
  }
}

pub async fn run_tmp() {
  dotenvy::dotenv().ok();

  tracing_subscriber::fmt::init();

  tracing::info!("Starting client...");

  let client = Client::new(ClientOptions {
    token: Credentials::Token(env::var("TOKEN").expect("TOKEN env var not set")),
    game: None,
    user_agent: None,
    social: None,
    ribbon: Some(RibbonOptions {
      options: Some(ribbon::Options {
        debug: true,
        logging: ribbon::LoggingLevel::Error,
        spooling: true,
      }),
      handling: None,
      transport: None,
      user_agent: None,
    }),
  })
  .await
  .expect("Failed to create client");

  tracing::info!("Client created: {:?}", client.user);

  let c = client.clone();
  tokio::select! {
    _ = async move {
    let client = c.clone();

    client
      .ribbon
      .on::<recv::client::DM>(async move |dm| {
        tracing::info!("Received DM from {}: {}", dm.username, dm.content);
      })
      .await;

    let invite = client
      .wait::<recv::social::Invite>()
      .await
      .expect("Failed to receive invite");

    tracing::info!("Received invite: {:?}", invite);

    client
      .join_room(&invite.roomid)
      .await
      .expect("Failed to join room");

    client.room().unwrap().switch(Bracket::Player).await.ok();

    tracing::info!(
      "Joined room {}, waiting for game to start...",
      invite.roomid
    );

    client
      .on::<recv::client::Dead>(|_| async {
        panic!("Connection closed permanently");
      })
      .await;

    let engine = Arc::new(parking_lot::Mutex::new(Falcon::new()));
    const PPS: f64 = 1.7;

    loop {
      client
        .ribbon
        .wait::<recv::client::game::round::Start>()
        .await
        .expect("Failed to receive game start event");

      {
        let mut engine_lock = engine.lock();
        let engine = if let Some(state) = client
          .game()
          .map(|g| g.me)
          .flatten()
          .map(|me| me.state)
        {
          state.lock().engine.clone()
        } else {
          tracing::error!("Failed to get initial game state (no engine available!");
          continue;
        };

        engine_lock.start(
          crate::engine::game::GameConfig {
            b2b_chaining: engine.initializer.b2b.chaining,
            b2b_charging: engine.initializer.b2b.charging.is_some(),
            b2b_charge_at: engine
              .initializer
              .b2b
              .charging
              .as_ref()
              .map(|v| v.at as i16)
              .unwrap_or(0),
            b2b_charge_base: engine
              .initializer
              .b2b
              .charging
              .as_ref()
              .map(|v| v.base as i16)
              .unwrap_or(0),
            combo_table: engine.initializer.options.combo_table,
            garbage_multiplier: engine.initializer.garbage.multiplier.value as f32,
            garbage_special_bonus: engine.initializer.garbage.special_bonus,
            kicks: match engine.initializer.kick_table {
              KickTable::SRSPlus => KickTable::SRSPlus,
              _ => {
                tracing::error!(
                  "Unsupported kick table: {:?}",
                  engine.initializer.kick_table
                );
                continue;
              }
            },
            pc_b2b: engine
              .initializer
              .pc
              .as_ref()
              .map(|pc| pc.b2b as u16)
              .unwrap_or(0),
            pc_send: engine
              .initializer
              .pc
              .as_ref()
              .map(|pc| pc.garbage as u16)
              .unwrap_or(0),
            spins: engine.initializer.options.spin_bonuses,
          },
          engine.queue.seed as u64,
          match engine.queue.kind {
            BagType::Bag7 => crate::engine::game::queue::Bag::Bag7,
            _ => {
              tracing::error!("Unsupported bag type: {:?}", engine.queue.kind);
              continue;
            }
          },
        );
      };

      let engine = engine.clone();
      let c = client.clone();

      client
        .register_ticker(move |input| {
          let engine = engine.clone();
          let client = c.clone();
          Box::pin(async move {
            let mut engine = engine.lock();
            if !input.new_garbage.is_empty() {
              engine.insert_garbage(
                input
                  .new_garbage
                  .iter()
                  .map(|g| crate::engine::game::Garbage {
                    col: g.column as u8,
                    amt: g.amount as u8,
                    time: 0,
                  })
                  .collect(),
              );
            }

            tick::Out {
              keys: if input.engine.frame % (60.0 / PPS) as u64 == 0 {
                let mv = engine.step(
                  input
                    .engine
                    .garbage_queue
                    .queue
                    .iter()
                    .map(|g| crate::engine::game::Garbage {
                      amt: g.amount as u8,
                      col: 0,
                      time: 0,
                    })
                    .collect(),
                );

                let mut keys: Vec<tick::Keypress> = Vec::new();
                let mut frame = FrameCounter::new(input.engine.frame);

                if let Some(res) = mv {
                  println!("{:?}", res.keys);
                  res.keys.iter().for_each(|k| {
                    let key = match k {
                      Move::Left => Key::MoveLeft,
                      Move::Right => Key::MoveRight,
                      Move::SoftDrop => Key::SoftDrop,
                      Move::HardDrop => Key::HardDrop,
                      Move::CW => Key::RotateCW,
                      Move::CCW => Key::RotateCCW,
                      Move::Flip => Key::Rotate180,
                      Move::Hold => Key::Hold,
                      Move::DasRight => Key::MoveRight,
                      Move::DasLeft => Key::MoveLeft,
                      Move::None => panic!("Move::None should not be in the move list"),
                    };

                    keys.push(tick::Keypress {
                      r#type: tick::KeypressType::Keydown,
                      frame: frame.frame(),
                      data: tick::KeypressData {
                        key,
                        subframe: frame.subframe(),
                        hoisted: false,
                      },
                    });

                    frame.add(if key == Key::SoftDrop {
                      0.1
                    } else if k == &Move::DasLeft || k == &Move::DasRight {
                      client.handling().das + 0.1
                    } else {
                      0.0
                    });

                    keys.push(tick::Keypress {
                      r#type: tick::KeypressType::Keyup,
                      frame: frame.frame(),
                      data: tick::KeypressData {
                        key,
                        subframe: frame.subframe(),
                        hoisted: false,
                      },
                    });
                  });
                }

                println!("{:?}", keys);

                keys
              } else {
                vec![]
              },
              run_after: vec![],
            }
          })
        })
        .await
        .ok();
    }
  } => {}
    _ = tokio::signal::ctrl_c() => {}
  }

  tracing::warn!("Shutting down...");
  client.destroy().await;
}
