use std::sync::Arc;

use crate::lib::commands::{CommandInfo, Commands, DefineParams, ListenerInput, Parameter};

use super::super::{Bot, Category, Restriction};

type Input = ListenerInput<Restriction, Arc<Bot>>;

pub fn register(cmds: &mut Commands<Restriction, Category, Arc<Bot>>) {
  cmds.define(
    &["help", "h"],
    DefineParams {
      description: "Why are you even looking at this?".into(),
      parameters: vec![
        Parameter {
          name: "command".into(),
          r#type: "string".into(),
          description: "The command to get help for (optional)".into(),
          optional: true,
        },
        Parameter {
          name: "parameter".into(),
          r#type: "string".into(),
          description: "The parameter to get help for (optional)".into(),
          optional: true,
        },
      ],
      category: Category::Info,
    },
    |input: Input| {
      let reply = input.reply;
      let args = input.args;
      let user_level = input.user.level;
      let bot = input.data;

      async move {
        let first_arg = args.first().cloned().unwrap_or_default();

        if first_arg.is_empty() {
          let category_priorities: &[(Category, u32)] = &[
            (Category::Info, 1),
            (Category::Controls, 2),
            (Category::Solver, 3),
            (Category::Dev, 99),
          ];

          let categorized = bot.commands.get_commands_by_category();

          let mut visible: Vec<(Category, u32, String)> = vec![];

          for (category, commands) in &categorized {
            if *category == Category::Dev && user_level != Restriction::Dev {
              continue;
            }
            if commands.is_empty() {
              continue;
            }
            let mut sorted = commands.clone();
            sorted.sort_by(|a, b| a.command.cmp(&b.command));
            let list = sorted
              .iter()
              .map(|c: &CommandInfo| {
                if c.alts.is_empty() {
                  c.command.clone()
                } else {
                  format!("{}/{}", c.command, c.alts.join("/"))
                }
              })
              .collect::<Vec<_>>()
              .join(" | ");

            let name = format!(
              "{}{}",
              &category_name(*category)[..1].to_uppercase(),
              &category_name(*category)[1..]
            );

            let priority = category_priorities
              .iter()
              .find(|(c, _)| c == category)
              .map(|(_, p)| *p)
              .unwrap_or(u32::MAX);

            visible.push((*category, priority, format!("\n{}:\n  {}", name, list)));
          }

          visible.sort_by_key(|(_, p, _)| *p);

          let output = visible
            .into_iter()
            .map(|(_, _, content)| content)
            .collect::<Vec<_>>()
            .join("\n");

          reply(format!("Available commands:{}", output));
          return;
        }

        let command_info = bot.commands.info(&first_arg).clone();
        let Some(c) = command_info else {
          reply("Command not found".into());
          return;
        };

        let second_arg = args.get(1).cloned().unwrap_or_default();
        if second_arg.is_empty() {
          let alts = if c.alts.is_empty() {
            String::new()
          } else {
            format!("({}) ", c.alts.join("/"))
          };
          let params = if c.parameters.is_empty() {
            String::new()
          } else {
            format!(
              "<{}>",
              c.parameters
                .iter()
                .map(|p| format!("{}{}", p.name, if p.optional { "?" } else { "" }))
                .collect::<Vec<_>>()
                .join(", ")
            )
          };
          reply(format!(
            "{} {}{}: {}",
            c.command, alts, params, c.description
          ));
        } else {
          let param = c.parameters.iter().find(|p| p.name == second_arg).cloned();
          match param {
            None => reply("Parameter not found".into()),
            Some(p) => reply(format!(
              "[{}] {} <{}>{}: {}",
              c.command,
              p.name,
              p.r#type,
              if p.optional { " (optional)" } else { "" },
              p.description
            )),
          }
        }
      }
    },
    false,
  );
}

fn category_name(c: Category) -> &'static str {
  match c {
    Category::Info => "info",
    Category::Controls => "controls",
    Category::Solver => "solver",
    Category::Dev => "dev",
  }
}
