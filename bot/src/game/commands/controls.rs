use std::sync::Arc;

use triangle::types::room::Bracket;

use crate::lib::commands::{Commands, DefineParams, ListenerInput, Parameter};

use super::super::{Bot, Category, Finesse, Restriction};

type Input = ListenerInput<Restriction, Arc<Bot>>;

const PPS_MIN: f64 = 0.5;
const PPS_MAX: f64 = 10.0;

pub fn register(cmds: &mut Commands<Restriction, Category, Arc<Bot>>) {
  cmds.define(
    &["kill"],
    DefineParams {
      description: "Kills the bot (from the room)".into(),
      parameters: vec![],
      category: Category::Controls,
    },
    |input: Input| {
      let reply = input.reply;
      let bot = input.data;
      async move {
        reply("bye :oyes:/".into());
        bot.destroy().await;
      }
    },
    true,
  );

  cmds.define(
    &["enable", "e"],
    DefineParams {
      description: "Move the bot to the player's bracket".into(),
      parameters: vec![Parameter {
        name: "force".into(),
        r#type: "force".into(),
        description: "Force enable (dev only)".into(),
        optional: true,
      }],
      category: Category::Controls,
    },
    |input: Input| {
      let reply = input.reply;
      let args = input.args;
      let user_level = input.user.level;
      let bot = input.data;
      async move {
        let enabled = bot.state.read().enabled.value;
        if enabled {
          reply("Gameplay is already enabled.".into());
          return;
        }
        let arg = args.first().cloned().unwrap_or_default();
        let force = arg == "force" && user_level == Restriction::Dev;
        if let Some(mut room) = bot.client.room() {
          match room.switch(Bracket::Player).await {
            Ok(_) => {
              {
                let mut state = bot.state.write();
                state.enabled.value = true;
                state.enabled.attempt = false;
                state.enabled.force = force;
              }
              reply("Enabled gameplay.".into());
            }
            Err(e) => {
              reply(format!("Error switching bracket: {}", e));
            }
          }
        } else {
          reply("Not in a room.".into());
        }
      }
    },
    true,
  );

  cmds.define(
    &["disable", "d"],
    DefineParams {
      description: "Move the bot to the spectators bracket".into(),
      parameters: vec![],
      category: Category::Controls,
    },
    |input: Input| {
      let reply = input.reply;
      let bot = input.data;
      async move {
        {
          let mut state = bot.state.write();
          state.enabled.attempt = false;
          state.enabled.force = false;
        }
        let enabled = bot.state.read().enabled.value;
        if !enabled {
          reply("Gameplay is already disabled.".into());
          return;
        }
        if let Some(mut room) = bot.client.room() {
          bot.state.write().enabled.value = false;
          match room.switch(Bracket::Spectator).await {
            Ok(_) => reply("Disabled gameplay.".into()),
            Err(_) => {
              reply("There was an error disabling gameplay, maybe it's already disabled?".into())
            }
          }
        } else {
          bot.state.write().enabled.value = false;
          reply("Disabled gameplay.".into());
        }
      }
    },
    true,
  );

  cmds.define(
    &["restrict"],
    DefineParams {
      description: "Restricts the bot to a certain level".into(),
      parameters: vec![Parameter {
        name: "level".into(),
        r#type: "none | player | host | dev".into(),
        description: "The level to restrict the bot to".into(),
        optional: false,
      }],
      category: Category::Controls,
    },
    |input: Input| {
      let reply = input.reply;
      let args = input.args;
      let user_level = input.user.level;
      let bot = input.data;
      async move {
        let Some(arg) = args.first().cloned() else {
          reply("Please provide a level.".into());
          return;
        };
        let v = match arg.as_str() {
          "none" => Restriction::None,
          "player" => Restriction::Player,
          "host" => Restriction::Host,
          "dev" => Restriction::Dev,
          _ => {
            reply(format!("Invalid restriction level: {}", arg));
            return;
          }
        };
        if user_level == Restriction::Player || user_level == Restriction::None {
          reply("Players cannot change restriction levels.".into());
          return;
        }
        if v == Restriction::Dev && user_level != Restriction::Dev {
          reply("This restriction level is locked to developers.".into());
          return;
        }
        bot.state.write().restriction = v;
        if v == Restriction::None {
          reply("Restrictions are now off.".into());
        } else {
          reply(format!("Restriction level now set to {}", arg));
        }
      }
    },
    true,
  );

  cmds.define(
    &["pps", "p"],
    DefineParams {
      description: "Set the bot's pps".into(),
      parameters: vec![Parameter {
        name: "speed".into(),
        r#type: "number".into(),
        description: "New bot pps".into(),
        optional: false,
      }],
      category: Category::Controls,
    },
    |input: Input| {
      let reply = input.reply;
      let args = input.args;
      let user_level = input.user.level;
      let bot = input.data;
      async move {
        let Some(arg) = args.first().cloned() else {
          let current = bot.config.read().pps;
          reply(format!("Current PPS: {}.", current));
          return;
        };
        let Ok(pps) = arg.parse::<f64>() else {
          reply("Invalid pps (must be a positive number)".into());
          return;
        };
        if pps <= 0.0 || pps.is_nan() {
          reply("Invalid pps (must be a positive number)".into());
          return;
        }
        let bypass = user_level == Restriction::Dev;
        if pps < PPS_MIN && !bypass {
          reply(format!("Invalid pps (less than {})", PPS_MIN));
          return;
        }
        if pps > PPS_MAX && !bypass {
          reply(format!("Invalid pps (greater than {}).", PPS_MAX));
          return;
        }
        let finesse = bot.config.read().finesse;
        if pps > 5.0 && finesse == Finesse::Smooth {
          reply(format!(
            "When finesse is enabled, PPS is capped to 5 PPS, run >finesse instant to unlock a maximum of {} PPS.",
            PPS_MAX
          ));
          return;
        }
        let rounded = (pps * 1000.0).round() / 1000.0;
        bot.config.write().pps = rounded;
        reply(format!("Set PPS to {}.", rounded));
      }
    },
    true,
  );

  cmds.define(
    &["finesse", "f"],
    DefineParams {
      description: "Set the bot's finesse mode".into(),
      parameters: vec![Parameter {
        name: "mode".into(),
        r#type: "smooth | instant".into(),
        description: "The finesse mode to use".into(),
        optional: false,
      }],
      category: Category::Controls,
    },
    |input: Input| {
      let reply = input.reply;
      let args = input.args;
      let bot = input.data;
      async move {
        let Some(arg) = args.first().cloned() else {
          let current = bot.config.read().finesse;
          let name = match current {
            Finesse::Smooth => "smooth",
            Finesse::Instant => "instant",
          };
          reply(format!("Current finesse mode: {}.", name));
          return;
        };
        let mode = match arg.as_str() {
          "smooth" => Finesse::Smooth,
          "instant" => Finesse::Instant,
          _ => {
            reply("Invalid finesse mode (must be 'smooth' or 'instant')".into());
            return;
          }
        };
        if mode == Finesse::Smooth {
          let pps = bot.config.read().pps;
          if pps > 5.0 {
            reply(
              "When switching to smooth finesse, PPS is capped to 5 PPS, run >pps 5 to comply."
                .into(),
            );
            return;
          }
        }
        bot.config.write().finesse = mode;
        reply(format!("Set finesse mode to {}.", arg));
      }
    },
    true,
  );
}
