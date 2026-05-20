use crate::bot::lib::env::{self, env};

pub mod bot;
pub mod engine;
pub mod io;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  dotenvy::dotenv().ok();
  env::parse_env();
  if env().server {
    io::start_server().await;
  } else {
    bot::run().await;
  }
}
