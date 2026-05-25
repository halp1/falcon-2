mod commands;
mod settings;
mod utils;
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

use crate::lib::{
  commands::{Commands, User},
  config::CONFIG,
  env::env,
  events::{events, msgs},
  logs::WSLogger,
};
use engine::{
  Falcon,
  game::{GameConfig, Garbage, data::Move, queue::Bag},
};
use settings::{ConstraintLevel, SettingsHandler};

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

  pub fn as_f64(&self) -> f64 {
    (self.0 * 10.0).round() / 10.0
  }

  pub fn max(&self, other: FrameCounter) -> Self {
    Self(self.0.max(other.0))
  }
}

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
  restriction: Restriction,
}

pub struct Bot {
  engine: Mutex<Falcon>,
  pub client: Client,
  pub config: RwLock<Config>,
  pub state: RwLock<State>,
  pub settings: SettingsHandler,
  events: EventEmitter,
  pub commands: Commands<Restriction, Category, Arc<Bot>>,
}

#[derive(Debug, Error)]
pub enum BotError {
  #[error("Failed to create client: {0}")]
  ConnectionError(ApiError),
  #[error("Failed to join or create room: {0}")]
  RoomError(WrapError),
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
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
          logging: ribbon::LoggingLevel::Error,
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

    let ws_logger = Arc::new(WSLogger::new()?);

    let ws_logger_2 = ws_logger.clone();

    client.on::<recv::client::ribbon::Receive>(async move |data| {
      ws_logger_2.push("receive", &data.command, &data.data);
    });

    client.on::<recv::client::ribbon::Send>(async move |data| {
      ws_logger.push("send", &data.command, &data.data);
    });

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

    let mut cmd = Commands::new(
      vec![
        Restriction::None,
        Restriction::Player,
        Restriction::Host,
        Restriction::Dev,
      ],
      vec![
        Category::Info,
        Category::Controls,
        Category::Solver,
        Category::Dev,
      ],
      ">",
      "",
      "falcon",
    );
    commands::register(&mut cmd);

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
        restriction: Restriction::None,
      }),
      events: EventEmitter::new(),
      commands: cmd,
    });

    bot.handle_room_update(room_update_data, true).await;

    if let Some(room) = bot.client.room() {
      room.chat(":oyes:/").await.ok();
    } else {
      return Err(BotError::RoomError(WrapError::ServerError));
    }

    bot.bind().await;

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
      b.handle_room_update(data, false).await;
    });

    let b = self.clone();

    self
      .client
      .on::<recv::client::game::round::End>(async move |_| {
        b.state.write().game = None;
      });

    let b = self.clone();

    self.client.on::<recv::room::Chat>(async move |data| {
      if data.system || data.user.id.is_none() {
        return;
      }
      if data
        .user
        .id
        .as_ref()
        .map_or(false, |id| *id == b.client.user.id)
      {
        return;
      }

      let bot_username = b.client.user.username.clone();
      let prefix = b.commands.prefix.clone();

      if data.content == format!("@{}", bot_username) {
        if let Some(room) = b.client.room() {
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
      let room_info = b.client.room().map(|r| {
        let s = r.state.lock();
        (s.owner.clone(), s.players.clone())
      });

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
        b.commands.prepare_calls(
          user,
          &content,
          b2.clone(),
          b.state.read().restriction,
          move |message| {
            let bb = b2.clone();
            let msg = message.clone();
            tracing::info!("sending message: {}", msg);
            tokio::spawn(async move {
              if let Some(room) = bb.client.room() {
                room.chat(&msg).await.ok();
              }
            });
          },
        )
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
      .on::<recv::client::game::Start>(async move |data| {
        if data.players.iter().any(|p| p.0 == b.client.user.id) {
          if let Some(room) = b.client.room() {
            room.chat("glhf!").await.ok();
          }
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

        b.client.game().unwrap().me.unwrap().set_pause_iges(true);

        let bag = match engine.queue.kind {
          BagType::Bag7 => Bag::Bag7,
          _ => {
            tracing::error!("Unsupported bag type: {:?}", engine.queue.kind);
            return;
          }
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
              garbage_cap: engine.initializer.garbage.cap.value as u16,
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
                .map(|pc| pc.garbage as u8)
                .unwrap_or(0),
              spins: engine.initializer.options.spin_bonuses,
              bag,
            },
            engine.queue.seed as u64,
            bag,
          );
        }

        {
          b.state.write().game = Some(GameState {
            last_piece_frame: 0,
            target_frame: 0,
          });
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

  async fn handle_room_update(self: &Arc<Self>, data: recv::room::Update, initial: bool) {
    let result = self.settings.check_room_update(&data);

    if let Some(result) = &result {
      if let Some(room) = self.client.room() {
        for output in &result.outputs {
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

        if initial
          && result
            .outputs
            .iter()
            .any(|o| o.message == "falcon requires 0 gravity increase.")
          && result
            .outputs
            .iter()
            .any(|o| o.message == "falcon requires 0 gravity.")
        {
          if let Some(room) = self.client.room() {
            room.chat("Paste:\n\n/set options.g=0;options.gincrease=0;\n\nin chat and press enter to enable falcon.").await.ok();
          }
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

  fn keypress_duration(&self, m: &Move, engine: &Engine) -> f64 {
    if m == &Move::SoftDrop {
      0.1
    } else if m == &Move::DasLeft || m == &Move::DasRight {
      engine.handling.das + 0.1
    } else {
      0.0
    }
  }

  fn process_keys(&self, raw: &[Move], engine: &Engine) -> Vec<tick::Keypress> {
    struct InternalKeypress {
      key: Key,
      frame: f64,
      duration: f64,
    }

    let now = engine.frame;

    let finesse = self.config.read().finesse;
    let frames: Vec<InternalKeypress> = match finesse {
      Finesse::Instant => {
        let mut frame = FrameCounter::new(now);
        raw
          .iter()
          .map(|m| {
            let duration = self.keypress_duration(m, engine);
            let kp = InternalKeypress {
              key: utils::move_to_key(*m),
              frame: frame.as_f64(),
              duration,
            };
            frame.add(duration);
            kp
          })
          .collect()
      }

      Finesse::Smooth => {
        const MAX_PIECE_FRAMES: u64 = 45;

        let mut frame = FrameCounter::new(now);
        let time_to_next = (self
          .next_piece_frame(engine, None)
          .saturating_sub(now)
          .saturating_sub(1))
        .min(MAX_PIECE_FRAMES);

        let arr = engine.handling.arr;

        let soft_drop_count = raw.iter().filter(|m| **m == Move::SoftDrop).count();
        let das_count = raw
          .iter()
          .filter(|m| **m == Move::DasLeft || **m == Move::DasRight)
          .count();
        let time_per_press = ((time_to_next as f64
          - soft_drop_count as f64 * 0.1
          - das_count as f64 * engine.handling.das)
          / raw.len() as f64)
          * 0.99;

        let mut sim_falling = engine.falling.clone();

        // key, frame, duration, delay
        let mut tmp: Vec<(Move, f64, f64, f64)> = Vec::new();

        for m in raw {
          let delay = time_per_press.max(0.0);
          let arr_time = if *m == Move::DasLeft || *m == Move::DasRight {
            let x_before = sim_falling.x();
            if *m == Move::DasLeft {
              sim_falling.das_left(&engine.board.state);
            } else {
              sim_falling.das_right(&engine.board.state);
            }
            let displacement = (sim_falling.x() - x_before).abs() as f64;
            (arr * (displacement - 1.0)).max(0.0)
          } else {
            match m {
              Move::CW => sim_falling.set_rotation(sim_falling.rotation() as i32 + 1),
              Move::CCW => sim_falling.set_rotation(sim_falling.rotation() as i32 - 1),
              Move::Flip => sim_falling.set_rotation(sim_falling.rotation() as i32 + 2),
              _ => {}
            }
            0.0
          };

          let duration = self.keypress_duration(m, engine) + arr_time;

          tmp.push((*m, frame.as_f64(), duration, delay));

          let prev_frame = frame.0;
          frame.add(delay + duration);

          if *m == Move::SoftDrop && frame.as_f64() != 0.0 {
            frame = frame.max(FrameCounter((prev_frame + duration).ceil()));
          }
        }

        let total: f64 = tmp.iter().map(|(_, _, d, delay)| delay + d).sum();
        if total > time_to_next as f64 {
          let duration_sum: f64 = tmp.iter().map(|(_, _, d, _)| d).sum();
          let multiplier = (time_to_next as f64 + duration_sum) / total;
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
        let mut frame = FrameCounter(f.frame);
        frame.add(0.0);

        let first = tick::Keypress {
          r#type: tick::KeypressType::Keydown,
          frame: frame.frame(),
          data: tick::KeypressData {
            key: f.key,
            subframe: frame.subframe(),
            hoisted: false,
          },
        };

        frame.add(f.duration);

        [
          first,
          tick::Keypress {
            r#type: tick::KeypressType::Keyup,
            frame: frame.frame(),
            data: tick::KeypressData {
              key: f.key,
              subframe: frame.subframe(),
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
        kp.data.subframe = (kp.data.subframe * 10.0).round() / 10.0;

        kp
      })
      .collect()
  }

  async fn tick(&self, input: tick::In) -> tick::Out {
    if !input.new_garbage.is_empty() {
      self.engine.lock().insert_garbage(
        input
          .new_garbage
          .iter()
          .map(|g| Garbage {
            col: g.column as u8,
            amt: g.amount as u16,
            time: 0,
          })
          .collect(),
      );
    }

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
        amt: g.amount as u16,
        col: 0,
        time: 0,
      })
      .collect();

    let mv = self.engine.lock().step(garbage_queue);

    tracing::info!(
      "keys: {:?}",
      mv.as_ref().map(|m| m.keys.clone()).unwrap_or_default()
    );

    let keys = if let Some(res) = mv {
      self.process_keys(&res.keys, &input.engine)
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
