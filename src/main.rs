use std::env;

pub mod bot;
pub mod engine;
pub mod io;

#[tokio::main]
async fn main() {
  if env::args().any(|arg| arg == "--server") {
		io::start_server().await;
	} else {
		bot::run().await;
	}
}
