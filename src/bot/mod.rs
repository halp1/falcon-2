pub mod game;
pub mod root;

pub async fn run() {
	game::run_tmp().await;
}