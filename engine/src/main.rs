#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub mod game;
// pub mod game2;
pub mod io;
pub mod keyfinder;
pub mod search;

#[tokio::main]
async fn main() {
  io::start_server().await;
}
