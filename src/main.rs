use bot::lib::env::{self, env};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  dotenvy::dotenv().ok();
  env::parse_env();
  if env().server {
    engine::io::start_server().await;
  } else {
    bot::run().await;
  }
}
