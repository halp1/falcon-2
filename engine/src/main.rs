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

pub mod game;
pub mod game2;
pub mod io;
pub mod keyfinder;
pub mod search;

#[tokio::main]
async fn main() {
  io::start_server().await;
}
