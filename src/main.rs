#![feature(
  test_incomplete_feature,
  adt_const_params,
	generic_const_exprs,
  inherent_associated_types,
  mgca_type_const_syntax,
  const_index,
  const_trait_impl,
  const_slice_make_iter,
  generic_const_items,
  portable_simd
)]

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
