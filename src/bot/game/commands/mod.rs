use std::sync::Arc;

use crate::bot::lib::commands::Commands;

use super::{Bot, Category, Restriction};

mod controls;
mod info;

pub fn register(commands: &mut Commands<Restriction, Category, Arc<Bot>>) {
  info::register(commands);
  controls::register(commands);
}
