use std::sync::Arc;

use super::Bot;

mod controls;
mod info;

pub fn register(bot: &Arc<Bot>) {
  info::register(bot);
  controls::register(bot);
}
