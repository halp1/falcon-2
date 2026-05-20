use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use thiserror::Error;
use triangle::{
  Client, ClientOptions, Engine,
  classes::ribbon,
  types::{
    events::recv,
    game::{Key, tick},
    room::Bracket,
  },
  utils::{EventEmitter, api::core::ApiError, events::WrapError},
};

use triangle::engine::queue::bag::BagType;

use crate::{
  bot::lib::{
    commands::{Commands, User},
    config::CONFIG,
    events::{events, msgs},
  },
  engine::{
    Falcon,
    game::{GameConfig, Garbage, data::Move, queue::Bag},
  },
  env,
};

mod commands;
mod settings;
mod utils;

use settings::{ConstraintLevel, SettingsHandler};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Restriction {
  None,
  Player,
  Host,
  Dev,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Category {
  Info,
  Controls,
  Solver,
  Dev,
}

#[derive(Debug, Clone)]
pub enum Target {
  Join(String),
  Create,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Finesse {
  Instant,
  Smooth,
}

#[derive(Debug, Clone)]
pub struct Config {
  pps: f64,
  finesse: Finesse,
}

#[derive(Debug, Clone)]
pub struct EnabledState {
  value: bool,
  attempt: bool,
  force: bool,
}

#[derive(Debug, Clone)]
pub struct GameState {
  last_piece_frame: u64,
  target_frame: u64,
}

#[derive(Debug, Clone)]
pub struct State {
  enabled: EnabledState,
  game: Option<GameState>,
}

pub struct Bot {
  engine: Mutex<Falcon>,
  pub client: Client,
  pub config: RwLock<Config>,
  pub state: RwLock<State>,
  pub settings: SettingsHandler,
  events: EventEmitter,
  pub commands: Mutex<Commands<Restriction, Category, Arc<Bot>>>,
}

#[derive(Debug, Error)]
pub enum BotError {
  #[error("Failed to create client: {0}")]
  ConnectionError(ApiError),
  #[error("Failed to join or create room: {0}")]
  RoomError(WrapError),
}

impl Bot {
  pub async fn new(target: Target) -> Result<Arc<Self>, BotError> {
    let client = Client::new(ClientOptions {
      game: Some(triangle::classes::GameOptions {
        handling: Some(CONFIG.handling),
        spectating_strategy: None,
      }),
      ribbon: Some(ribbon::OptionalParams {
        options: Some(ribbon::Options {
          logging: ribbon::LoggingLevel::All,
          ..Default::default()
        }),
        ..Default::default()
      }),
      social: None,
      token: triangle::Credentials::Token(env().token.clone()),
      user_agent: None,
    })
    .await
    .map_err(BotError::ConnectionError)?;


    let (room_tx, room_rx) = tokio::sync::oneshot::channel::<recv::room::Update>();
    let room_tx = Arc::new(Mutex::new(Some(room_tx)));

    client.on::<recv::room::Update>(async move |data| {
      if let Some(tx) = room_tx.lock().take() {
        tx.send(data).ok();
      }
    });

    match target {
      Target::Join(roomid) => client.join_room(&roomid).await,
      Target::Create => client.create_room(false).await,
    }
    .map_err(BotError::RoomError)?;

    let room_update_data = room_rx
      .await
      .map_err(|_| BotError::RoomError(WrapError::ServerError))?;

    let bot = Arc::new(Bot {
      engine: Mutex::new(Falcon::new()),
      client,
      settings: SettingsHandler::new(),
      config: RwLock::new(Config {
        finesse: Finesse::Smooth,
        pps: 1.0,
      }),
      state: RwLock::new(State {
        enabled: EnabledState {
          value: false,
          attempt: true,
          force: false,
        },
        game: None,
      }),
      events: EventEmitter::new(),
      commands: Mutex::new(Commands::new(
        vec![
          Restriction::None,
          Restriction::Player,
          Restriction::Host,
          Restriction::Dev,
        ],
        Restriction::None,
        vec![
          Category::Info,
          Category::Controls,
          Category::Solver,
          Category::Dev,
        ],
        ".",
        "",
        "falcon",
        Some(Restriction::Dev),
      )),
    });


    bot.handle_room_update(room_update_data).await;


    if let Some(mut room) = bot.client.room() {
      room.chat(":oyes:/").await.ok();
    } else {
      return Err(BotError::RoomError(WrapError::ServerError));
    }


    bot.bind().await;
    commands::register(&bot);
    bot.commands.lock().restrict(Restriction::None);

    Ok(bot)
  }

  async fn bind(self: &Arc<Self>) {
    let b = self.clone();
    events()
      .on::<msgs::Shutdown>(async move |_| {
        let b = b.clone();
        b.destroy().await;
      })
      .await;

    let b = self.clone();

    self.client.on::<recv::client::Dead>(async move |_| {
      b.destroy().await;
    });

    let b = self.clone();

    self.client.on::<recv::room::Leave>(async move |_| {
      b.destroy().await;
    });

    let b = self.clone();

    self.client.on::<recv::room::Update>(async move |data| {
      b.handle_room_update(data).await;
    });

    let b = self.clone();

    self
      .client
      .on::<recv::client::game::round::End>(async move |_| {
        b.state.write().game = None;
      });

    let b = self.clone();

    self.client.on::<recv::room::Chat>(async move |data| {
      if data.user.id.is_none() {
        return;
      }
      if data.user.username == b.client.user.username {
        return;
      }

      let bot_username = b.client.user.username.clone();
      let prefix = b.commands.lock().prefix.clone();

      if data.content == format!("@{}", bot_username) {
        if let Some(mut room) = b.client.room() {
          room.chat(&format!("My prefix is {}", prefix)).await.ok();
        }
        return;
      }

      let content = if data.content.starts_with(&format!("@{} ", bot_username)) {
        data
          .content
          .replacen(&format!("@{} ", bot_username), &prefix, 1)
      } else {
        data.content.clone()
      }
      .to_lowercase();

      let user_id = data.user.id.as_deref().unwrap_or("").to_string();
      let room_info = b
        .client
        .room()
        .map(|r| (r.state.lock().owner.clone(), r.state.lock().players.clone()));

      let level = if let Some((owner, players)) = &room_info {
        if user_id == *owner {
          Restriction::Host
        } else if players
          .iter()
          .any(|p| matches!(p.bracket, Bracket::Player) && p.id == user_id)
        {
          Restriction::Player
        } else {
          Restriction::None
        }
      } else {
        Restriction::None
      };

      let user = User {
        id: user_id,
        name: data.user.username.clone(),
        level,
      };

      let b2 = b.clone();
      let futures = {
        let mut cmds = b.commands.lock();
        cmds.prepare_calls(user, &content, b2.clone(), move |message| {
          let bb = b2.clone();
          let msg = message.clone();
					tracing::info!("sending message: {}", msg);
          tokio::spawn(async move {
            if let Some(mut room) = bb.client.room() {
              room.chat(&msg).await.ok();
            }
          });
        })
      };

      for fut in futures {
        fut.await;
      }
    });

    // this.client.on("client.room.players", (players) => {
    //   if (players.every((p) => p.bot)) this.destroy();
    // });

    let b = self.clone();

    self
      .client
      .on::<recv::client::room::Players>(async move |data| {
        if data.0.iter().all(|p| p.bot) {
          b.destroy().await;
        }
      });

    let b = self.clone();

    self
      .client
      .on::<recv::client::game::round::Start>(async move |_| {
        let engine_snap = b
          .client
          .game()
          .and_then(|g| g.me)
          .map(|me| me.state.lock().engine.clone());

        let engine = match engine_snap {
          Some(e) => e,
          None => return,
        };

        {
          let mut falcon = b.engine.lock();
          falcon.start(
            GameConfig {
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
              kicks: engine.initializer.kick_table,
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
              BagType::Bag7 => Bag::Bag7,
              _ => {
                tracing::error!("Unsupported bag type: {:?}", engine.queue.kind);
                return;
              }
            },
          );
        }

        {
          let target_frame = b.next_piece_frame(&engine, None);
          b.state.write().game = Some(GameState {
            last_piece_frame: engine.frame,
            target_frame,
          });
        }

        let b2 = b.clone();
        b.client
          .register_ticker(move |input| {
            let b = b2.clone();
            Box::pin(async move { b.tick(input).await })
          })
          .await
          .ok();
      });
  }

  async fn handle_room_update(self: &Arc<Self>, data: recv::room::Update) {
    let result = self.settings.check_room_update(&data);
    if let Some(result) = &result {
      for output in &result.outputs {
        if let Some(mut room) = self.client.room() {
          room
            .chat(&format!(
              "{}: {}",
              output.level.to_string().to_uppercase(),
              output.message
            ))
            .await
            .ok();
        }
      }
      if result.level == ConstraintLevel::Error {
        if let Some(mut room) = self.client.room() {
          room.switch(Bracket::Spectator).await.ok();
        }
        {
          let mut state = self.state.write();
          state.enabled.attempt = true;
          state.enabled.value = false;
        }
        return;
      }
    }
    let attempt = self.state.read().enabled.attempt;
    if result
      .as_ref()
      .map_or(true, |r| r.level != ConstraintLevel::Error)
      && attempt
    {
      if let Some(mut room) = self.client.room() {
        room.switch(Bracket::Player).await.ok();
      }
      self.state.write().enabled.value = true;
    }
  }

  fn next_piece_frame(&self, engine: &Engine, next_hard_drop_frame: Option<f64>) -> u64 {
    const MAX_DELTA: f64 = 0.2;
    let pps = self.config.read().pps;
    let last_piece_frame = {
      let state = self.state.read();
      state
        .game
        .as_ref()
        .map_or(engine.frame as f64, |g| g.last_piece_frame as f64)
    };

    let frames = utils::frames_till_next_piece(
      engine.stats.pieces,
      pps,
      last_piece_frame,
      pps * (1.0 - MAX_DELTA),
      pps * (1.0 + MAX_DELTA),
    );

    let result = utils::normal_random(frames, 1.0) + last_piece_frame;
    let next_hd = next_hard_drop_frame.unwrap_or(f64::NEG_INFINITY) + 1.0;

    result.max(next_hd).max(engine.frame as f64 + 1.0) as u64
  }

  fn process_keys(&self, raw: &[Move], engine: &Engine) -> Vec<tick::Keypress> {
    struct InternalKeypress {
      key: Key,
      frame: f64,
      duration: f64,
    }

    let now = engine.frame as f64;

    let finesse = self.config.read().finesse;
    let frames: Vec<InternalKeypress> = match finesse {
      Finesse::Instant => {
        let mut frame = now;
        raw
          .iter()
          .map(|m| {
            let duration = if *m == Move::SoftDrop {
              0.1
            } else if *m == Move::DasLeft || *m == Move::DasRight {
              engine.handling.das
            } else {
              0.0
            };
            let kp = InternalKeypress {
              key: utils::move_to_key(*m),
              frame,
              duration,
            };
            frame += duration;
            kp
          })
          .collect()
      }

      Finesse::Smooth => {
        const MAX_PIECE_FRAMES: f64 = 45.0;

        let mut running_frame = now;
        let time_to_next =
          (self.next_piece_frame(engine, None) as f64 - now - 1.0).min(MAX_PIECE_FRAMES);

        let soft_drop_count = raw.iter().filter(|m| **m == Move::SoftDrop).count();
        let time_per_press =
          ((time_to_next - soft_drop_count as f64 * 0.1) / raw.len() as f64) * 0.99;

        let mut tmp: Vec<(Move, f64, f64, f64)> = Vec::new();

        for m in raw {
          let delay = time_per_press.max(0.0);
          let duration = if *m == Move::SoftDrop { 0.1 } else { 0.0 };

          tmp.push((*m, running_frame, duration, delay));

          let prev_frame = running_frame;
          running_frame += delay + duration;

          if *m == Move::SoftDrop && running_frame % 1.0 != 0.0 {
            running_frame = running_frame.max((prev_frame + duration).ceil());
          }
        }

        let total: f64 = tmp.iter().map(|(_, _, d, delay)| delay + d).sum();
        if total > time_to_next {
          let duration_sum: f64 = tmp.iter().map(|(_, _, d, _)| d).sum();
          let multiplier = (time_to_next + duration_sum) / total;
          tmp
            .iter_mut()
            .for_each(|(_, _, _, delay)| *delay *= multiplier);
        }

        tmp
          .into_iter()
          .map(|(m, f, d, _)| InternalKeypress {
            key: utils::move_to_key(m),
            frame: f,
            duration: d,
          })
          .collect()
      }
    };

    frames
      .into_iter()
      .flat_map(|f| {
        [
          tick::Keypress {
            r#type: tick::KeypressType::Keydown,
            frame: f.frame.floor() as u64,
            data: tick::KeypressData {
              key: f.key,
              subframe: ((f.frame % 1.0) * 10.0).round() / 10.0,
              hoisted: false,
            },
          },
          tick::Keypress {
            r#type: tick::KeypressType::Keyup,
            frame: (f.frame + f.duration).floor() as u64,
            data: tick::KeypressData {
              key: f.key,
              subframe: (((f.frame + f.duration) % 1.0) * 10.0).round() / 10.0,
              hoisted: false,
            },
          },
        ]
      })
      .map(|mut kp| {
        while kp.data.subframe >= 1.0 {
          kp.data.subframe -= 1.0;
          kp.frame += 1;
        }
        kp
      })
      .collect()
  }

  async fn tick(&self, input: tick::In) -> tick::Out {
    let game_state = { self.state.read().game.as_ref().map(|g| g.target_frame) };

    let Some(target_frame) = game_state else {
      return tick::Out {
        keys: vec![],
        run_after: vec![],
      };
    };

    if input.engine.frame < target_frame {
      return tick::Out {
        keys: vec![],
        run_after: vec![],
      };
    }

    let has_hard_drop = self
      .client
      .game()
      .and_then(|g| g.me)
      .map(|me| {
        me.state
          .lock()
          .key_queue
          .iter()
          .any(|kp| kp.data.key == Key::HardDrop)
      })
      .unwrap_or(false);

    if has_hard_drop {
      return tick::Out {
        keys: vec![],
        run_after: vec![],
      };
    }

    if !input.new_garbage.is_empty() {
      self.engine.lock().insert_garbage(
        input
          .new_garbage
          .iter()
          .map(|g| Garbage {
            col: g.column as u8,
            amt: g.amount as u8,
            time: 0,
          })
          .collect(),
      );
    }

    {
      let mut state = self.state.write();
      if let Some(game) = &mut state.game {
        game.last_piece_frame = input.engine.frame;
      }
    }

    let initial_target = self.next_piece_frame(&input.engine, None);
    {
      let mut state = self.state.write();
      if let Some(game) = &mut state.game {
        game.target_frame = initial_target;
      }
    }

    let garbage_queue: Vec<Garbage> = input
      .engine
      .garbage_queue
      .queue
      .iter()
      .map(|g| Garbage {
        amt: g.amount as u8,
        col: 0,
        time: 0,
      })
      .collect();

    let mv = self.engine.lock().step(garbage_queue);

    let keys = if let Some(res) = mv {
      let mut moves = res.keys.clone();
      moves.push(Move::HardDrop);
      self.process_keys(&moves, &input.engine)
    } else {
      vec![]
    };

    let hd_frame = keys
      .iter()
      .rev()
      .find(|kp| kp.data.key == Key::HardDrop)
      .map(|kp| kp.frame as f64);

    let final_target = self.next_piece_frame(&input.engine, hd_frame);
    {
      let mut state = self.state.write();
      if let Some(game) = &mut state.game {
        game.target_frame = final_target;
      }
    }

    tick::Out {
      keys,
      run_after: vec![],
    }
  }

  async fn destroy(&self) {
    self.client.destroy().await;

    self.events.emit_raw("close", serde_json::json!({}));
    self.events.destroy();
  }
}
