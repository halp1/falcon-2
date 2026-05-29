use super::board::Board;

use super::data::Mino;

const fn real_permutations<const PIECE: Mino>() -> usize {
  PIECE.real_permuations()
}

pub struct CollisionMap<
  const PIECE: Mino,
  const WIDTH: usize,
  const HEIGHT: usize,
  const BUFFER: usize,
> where
  [(); real_permutations::<PIECE>()]:,
{
  data: [Board<WIDTH, HEIGHT, BUFFER>; real_permutations::<PIECE>()],
}

impl<
		const PIECE: Mino,
		const WIDTH: usize,
		const HEIGHT: usize,
		const BUFFER: usize,
	> CollisionMap<PIECE, WIDTH, HEIGHT, BUFFER>
where
	[(); real_permutations::<PIECE>()]:,
{
	pub const fn usable() -> Self {
		let mut result = Self {
			data: [Board::new(); real_permutations::<PIECE>()],
		};

		
	}
}
