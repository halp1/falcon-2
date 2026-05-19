use crate::bot::lib::events::{events, msgs};

pub mod game;
pub mod lib;
pub mod root;

pub async fn run() {
  let master = root::master::Master::new().await;
  tokio::signal::ctrl_c().await;
  events().emit(msgs::Shutdown).await;
}
