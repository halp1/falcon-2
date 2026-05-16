fn main() {
  futures::executor::block_on(falcon_2::protocol::start_server());
}
