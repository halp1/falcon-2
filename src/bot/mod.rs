use crate::bot::lib::events::{events, msgs};

pub mod game;
pub mod lib;
pub mod root;

pub async fn run() {
  root::master::Master::new().await.expect("Failed to start master client");
	tracing::info!("Master client created");
  tokio::signal::ctrl_c().await.ok();
  events().emit(msgs::Shutdown).await;
}
