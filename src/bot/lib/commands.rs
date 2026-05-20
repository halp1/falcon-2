use std::sync::Arc;
use std::collections::HashMap;

use futures::future::BoxFuture;
use triangle::utils::events::AsyncCallback;

pub use triangle::utils::events::SyncFn;

pub struct User<L> {
  pub id: String,
  pub name: String,
  pub level: L,
}

pub struct ListenerInput<L, D = ()> {
  pub reply: Box<dyn Fn(String) + Send + Sync>,
  pub args: Vec<String>,
  pub user: User<L>,
  pub data: D,
}

#[derive(Clone)]
pub struct Parameter {
  pub name: String,
  pub r#type: String,
  pub description: String,
  pub optional: bool,
}

#[derive(Clone)]
pub struct CommandInfo {
  pub command: String,
  pub alts: Vec<String>,
  pub description: String,
  pub parameters: Vec<Parameter>,
}

struct CommandEntry<L, C, D> {
  command: String,
  alts: Vec<String>,
  listener: Arc<dyn Fn(ListenerInput<L, D>) -> BoxFuture<'static, ()> + Send + Sync>,
  restricted: bool,
  description: String,
  category: C,
  parameters: Vec<Parameter>,
}

pub struct DefineParams<C> {
  pub description: String,
  pub parameters: Vec<Parameter>,
  pub category: C,
}

pub struct DefineOptions<L> {
  pub restricted: bool,
}

impl<L> Default for DefineOptions<L> {
  fn default() -> Self {
    Self {
      restricted: false,
    }
  }
}

pub struct Commands<L, C, D = ()> {
  pub prefix: String,
  pub reply_prefix: String,
  pub name: String,
  restriction: L,
  restriction_levels: Vec<L>,
  categories: Vec<C>,
  listeners: Vec<CommandEntry<L, C, D>>,
}

impl<L, C, D> Commands<L, C, D>
where
  L: Eq + PartialOrd + Clone + Send + Sync + 'static,
  C: Eq + Clone + Send + Sync + 'static,
  D: Clone + Send + Sync + 'static,
{
  pub fn new(
    restriction_levels: Vec<L>,
    default_restriction: L,
    categories: Vec<C>,
    prefix: impl Into<String>,
    reply_prefix: impl Into<String>,
    name: impl Into<String>,
  ) -> Self {
    Self {
      prefix: prefix.into(),
      reply_prefix: reply_prefix.into(),
      name: name.into(),
      restriction: default_restriction,
      restriction_levels,
      categories,
      listeners: Vec::new(),
    }
  }

  pub fn define<F>(
    &mut self,
    commands: &[&str],
    params: DefineParams<C>,
    listener: F,
    options: DefineOptions<L>,
  ) where
    F: AsyncCallback<ListenerInput<L, D>> + Sync,
  {
    let cooldown = options.cooldown.map(|c| Cooldown {
      value: c.value,
      bypass: c.bypass.or_else(|| self.default_cooldown_bypass.clone()),
      players: HashMap::new(),
    });

    let arced: Arc<dyn Fn(ListenerInput<L, D>) -> BoxFuture<'static, ()> + Send + Sync> =
      Arc::new(move |input| Box::pin(listener.clone().call(input)));

    self.listeners.push(CommandEntry {
      command: commands[0].to_string(),
      alts: commands[1..].iter().map(|s| s.to_string()).collect(),
      listener: arced,
      restricted: options.restricted,
      cooldown,
      description: params.description,
      category: params.category,
      parameters: params.parameters,
    });
  }

  pub fn off(&mut self, command: &str) {
    self.listeners.retain(|l| l.command != command);
  }

  pub fn commands(&self) -> Vec<Vec<String>> {
    self
      .listeners
      .iter()
      .map(|l| {
        std::iter::once(l.command.clone())
          .chain(l.alts.clone())
          .collect()
      })
      .collect()
  }

  pub fn set_prefix(&mut self, prefix: impl Into<String>) {
    self.prefix = prefix.into();
  }

  pub fn info(&self, command: &str) -> Option<CommandInfo> {
    self
      .listeners
      .iter()
      .find(|l| l.command == command || l.alts.iter().any(|a| a == command))
      .map(|l| CommandInfo {
        command: l.command.clone(),
        alts: l.alts.clone(),
        description: l.description.clone(),
        parameters: l
          .parameters
          .iter()
          .map(|p| Parameter {
            name: p.name.clone(),
            r#type: p.r#type.clone(),
            description: p.description.clone(),
            optional: p.optional,
          })
          .collect(),
      })
  }

  pub fn get_commands_by_category(&self) -> HashMap<C, Vec<CommandInfo>>
  where
    C: std::hash::Hash,
  {
    let mut map: HashMap<C, Vec<CommandInfo>> = self
      .categories
      .iter()
      .map(|c| (c.clone(), Vec::new()))
      .collect();

    for l in &self.listeners {
      if let Some(entries) = map.get_mut(&l.category) {
        entries.push(CommandInfo {
          command: l.command.clone(),
          alts: l.alts.clone(),
          description: l.description.clone(),
          parameters: l
            .parameters
            .iter()
            .map(|p| Parameter {
              name: p.name.clone(),
              r#type: p.r#type.clone(),
              description: p.description.clone(),
              optional: p.optional,
            })
            .collect(),
        });
      }
    }

    map.retain(|_, v| !v.is_empty());
    map
  }

  pub fn prepare_calls(
    &mut self,
    user: User<L>,
    message: &str,
    data: D,
    send_message: impl Fn(String) + Send + Sync + Clone + 'static,
  ) -> Vec<BoxFuture<'static, ()>> {
    if !message.starts_with(&self.prefix) {
      return vec![];
    }

    let suffix = &message[self.prefix.len()..];
    let mut parts = suffix.splitn(2, ' ');
    let command = match parts.next() {
      Some(c) if !c.is_empty() => c.to_lowercase(),
      _ => return vec![],
    };
    let args: Vec<String> = parts
      .next()
      .unwrap_or("")
      .split(' ')
      .filter(|s| !s.is_empty())
      .map(|s| s.to_string())
      .collect();

    let matching: Vec<usize> = self
      .listeners
      .iter()
      .enumerate()
      .filter(|(_, l)| {
        l.command.to_lowercase() == command || l.alts.iter().any(|a| a.to_lowercase() == command)
      })
      .map(|(i, _)| i)
      .collect();

    if matching.is_empty() {
      let sm = send_message;
      let prefix = self.prefix.clone();
      return vec![Box::pin(async move {
        sm(format!(
          "Unknown command.\nRun {}help for a list of valid commands.",
          prefix
        ));
      })];
    }

    let user_level_idx = self
      .restriction_levels
      .iter()
      .position(|r| r == &user.level)
      .unwrap_or(0);

    let required_idx = self
      .restriction_levels
      .iter()
      .position(|r| r == &self.restriction)
      .unwrap_or(0);

    let mut futures: Vec<BoxFuture<'static, ()>> = vec![];

    for idx in matching {
      if self.listeners[idx].restricted && user_level_idx < required_idx {
        let sm = send_message.clone();
        let reply_prefix = self.reply_prefix.clone();
        let name = self.name.clone();
        futures.push(Box::pin(async move {
          sm(format!(
            "{}{}'s commands are currently restricted.",
            reply_prefix, name
          ));
        }));
        continue;
      }

      let bypass_idx = self.listeners[idx]
        .cooldown
        .as_ref()
        .and_then(|cd| cd.bypass.as_ref())
        .and_then(|b| self.restriction_levels.iter().position(|r| r == b));

      let should_check_cooldown = self.listeners[idx].cooldown.is_some()
        && bypass_idx.map(|bi| user_level_idx < bi).unwrap_or(true);

      if should_check_cooldown {
        let skip_msg = self.listeners[idx].cooldown.as_ref().and_then(|cd| {
          cd.players.get(&user.id).and_then(|&last_used| {
            if last_used.elapsed() < cd.value {
              let remaining = cd.value.saturating_sub(last_used.elapsed());
              Some(format!(
                "{}You must wait {} seconds before using this command again.",
                self.reply_prefix,
                remaining.as_secs_f64().ceil() as u64
              ))
            } else {
              None
            }
          })
        });

        if let Some(msg) = skip_msg {
          let sm = send_message.clone();
          futures.push(Box::pin(async move {
            sm(msg);
          }));
          continue;
        }

        self.listeners[idx]
          .cooldown
          .as_mut()
          .unwrap()
          .players
          .insert(user.id.clone(), Instant::now());
      }

      let reply_prefix = self.reply_prefix.clone();
      let send_clone = send_message.clone();
      let reply: Box<dyn Fn(String) + Send + Sync> =
        Box::new(move |msg| send_clone(format!("{}{}", reply_prefix, msg)));

      let input = ListenerInput {
        reply,
        args: args.clone(),
        user: User {
          id: user.id.clone(),
          name: user.name.clone(),
          level: user.level.clone(),
        },
        data: data.clone(),
      };

      let listener = Arc::clone(&self.listeners[idx].listener);
      futures.push((listener.as_ref())(input));
    }

    futures
  }

  pub fn restrict(&mut self, level: L) {
    self.restriction = level;
  }

  pub fn level(&self) -> &L {
    &self.restriction
  }
}
