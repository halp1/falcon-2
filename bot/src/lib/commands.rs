use std::collections::HashMap;
use std::sync::Arc;

use futures::future::BoxFuture;
use triangle::utils::events::AsyncCallback;

pub use triangle::utils::events::SyncFn;

#[derive(Debug)]
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

pub struct Commands<L, C, D = ()> {
  pub prefix: String,
  pub reply_prefix: String,
  pub name: String,
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
    categories: Vec<C>,
    prefix: impl Into<String>,
    reply_prefix: impl Into<String>,
    name: impl Into<String>,
  ) -> Self {
    Self {
      prefix: prefix.into(),
      reply_prefix: reply_prefix.into(),
      name: name.into(),
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
    restricted: bool,
  ) where
    F: AsyncCallback<ListenerInput<L, D>> + Sync,
  {
    let arced: Arc<dyn Fn(ListenerInput<L, D>) -> BoxFuture<'static, ()> + Send + Sync> =
      Arc::new(move |input| Box::pin(listener.clone().call(input)));

    self.listeners.push(CommandEntry {
      command: commands[0].to_string(),
      alts: commands[1..].iter().map(|s| s.to_string()).collect(),
      listener: arced,
      restricted,
      description: params.description,
      category: params.category,
      parameters: params.parameters,
    });
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
    &self,
    user: User<L>,
    message: &str,
    data: D,
    restriction: L,
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
      .position(|r| r == &restriction)
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
}
