use bot::lib::env::{self, env};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  dotenvy::dotenv().ok();
  env::parse_env();

	// ensure weights file exists
	let weights_path = env().weights.clone();
	if !std::path::Path::new(&weights_path).exists() {
		eprintln!("Weights file not found at '{}'", weights_path);
		std::process::exit(1);
	}

  if env().server {
    engine::io::start_server().await;
  } else {
    bot::run().await;
  }
}
