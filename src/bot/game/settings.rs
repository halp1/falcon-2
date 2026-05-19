use triangle::{engine::{queue::bag::BagType, utils::KickTable}, types::{
  events::recv,
  game::{GarbageEntry, GarbageTargetBonus, Passthrough, SpinBonuses},
}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstraintLevel {
  Info,
  Change,
  Warning,
  Error,
}

impl std::fmt::Display for ConstraintLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Info => write!(f, "info"),
      Self::Change => write!(f, "change"),
      Self::Warning => write!(f, "warning"),
      Self::Error => write!(f, "error"),
    }
  }
}

pub struct ConstraintOutput {
  pub level: ConstraintLevel,
  pub message: String,
}

pub struct CheckResult {
  pub level: ConstraintLevel,
  pub outputs: Vec<ConstraintOutput>,
}

type Constraint = Box<dyn Fn(&recv::room::Update) -> Option<ConstraintOutput> + Send + Sync>;

fn err(msg: &str) -> Option<ConstraintOutput> {
  Some(ConstraintOutput {
    level: ConstraintLevel::Error,
    message: msg.to_string(),
  })
}

pub struct SettingsHandler {
  constraints: Vec<Constraint>,
}

impl SettingsHandler {
  pub fn new() -> Self {
    Self {
      constraints: Self::default_constraints(),
    }
  }

  fn default_constraints() -> Vec<Constraint> {
    vec![
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.spinbonuses.as_ref())
          .map_or(false, |v| *v == SpinBonuses::AllMini);
        if !ok {
          err("falcon does not support spin bonuses outside of all-mini.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.passthrough.as_ref())
          .map_or(false, |v| {
            matches!(v, Passthrough::Zero)
          });
        if !ok {
          err("falcon only supports zero passthrough.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.kickset.as_ref())
          .map_or(false, |v| matches!(v, KickTable::SRSPlus));
        if !ok {
          err(r#"falcon only supports the "SRS+" kick table."#)
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.allow_harddrop)
          .unwrap_or(false);
        if !ok {
          err("falcon requires hard drop to be enabled in order to play.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.are)
          .map_or(false, |v| v == 0);
        if !ok {
          err(r#"falcon only supports "0" ARE."#)
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.lineclear_are)
          .map_or(false, |v| v == 0);
        if !ok {
          err(r#"falcon only supports "0" line clear ARE."#)
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.room_handling)
          .map_or(true, |v| !v);
        if !ok {
          err("falcon does not support custom room handling.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.boardwidth)
          .map_or(false, |v| v == 10);
        if !ok {
          err("falcon currently only supports boards with a width of 10.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.locktime)
          .map_or(false, |v| v > 1);
        if !ok {
          err("falcon requires at least 1 lock delay.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.nolockout)
          .unwrap_or(false);
        if !ok {
          err("falcon does not support lockout.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.stock)
          .map_or(false, |v| v == 0);
        if !ok {
          err("falcon does not support stock, but will soon.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let gamemode = &data.r#match.gamemode;
        let ok = gamemode == "versus" || gamemode == "practice";
        if !ok {
          err("falcon does not properly support royale mode.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.garbagephase)
          .map_or(false, |v| v == 0);
        if !ok {
          err("falcon does not yet support garbage phasing.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.garbageentry.as_ref())
          .map_or(false, |v| *v == GarbageEntry::Instant);
        if !ok {
          err("falcon does not support non-instant garbage entry.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.garbagequeue)
          .map_or(true, |v| !v);
        if !ok {
          err("falcon does not support garbage queue.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.messiness_timeout)
          .map_or(false, |v| v == 0.0);
        if !ok {
          err("falcon does not support messiness timeout.")
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.bagtype.as_ref())
          .map_or(true, |v| matches!(v, BagType::Bag7));
        if !ok {
          err(r#"falcon does not support the "classic" or "total mayhem" bag types."#)
        } else {
          None
        }
      }),
      Box::new(|data| {
        let ok = data
          .options
          .as_ref()
          .and_then(|o| o.garbagetargetbonus.as_ref())
          .map_or(false, |v| *v == GarbageTargetBonus::None);
        if !ok {
          err("falcon does not support garbage targeting bonuses.")
        } else {
          None
        }
      }),
    ]
  }

  pub fn check_room_update(&self, data: &recv::room::Update) -> Option<CheckResult> {
    let outputs: Vec<ConstraintOutput> = self.constraints.iter().filter_map(|c| c(data)).collect();

    if outputs.is_empty() {
      return None;
    }

    let level = outputs
      .iter()
      .fold(outputs[0].level, |acc, o| acc.max(o.level));

    Some(CheckResult { level, outputs })
  }
}
