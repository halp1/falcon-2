pub mod game;
pub mod io;
pub mod keyfinder;
pub mod search;
pub mod trainer;

#[tokio::main]
async fn main() {
  io::start_server().await;
}
