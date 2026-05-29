#[derive(Copy, Clone)]
pub struct Board<const WIDTH: usize, const HEIGHT: usize, const BUFFER: usize> {
  data: [u64; WIDTH],
}

impl<const WIDTH: usize, const HEIGHT: usize, const BUFFER: usize> Board<WIDTH, HEIGHT, BUFFER> {
  pub const fn width(&self) -> usize {
    WIDTH
  }
  pub const fn height(&self) -> usize {
    HEIGHT
  }
  pub const fn buffer(&self) -> usize {
    BUFFER
  }

  pub const fn new() -> Self {
    assert!(HEIGHT <= 64, "board height max is 64");
    Self { data: [0; WIDTH] }
  }
}
