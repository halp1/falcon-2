use triangle::types::game::{Buffering, Handling};

pub struct Config {
	pub handling: Handling,
}

pub const CONFIG: Config = Config {
	handling: Handling {
		arr: 0.0,
		das: 6.0,
		ihs: Buffering::Tap,
		irs: Buffering::Tap,
		dcd: 0.0,
		sdf: 41.0,
		safelock: false,
		cancel: false,
		may20g: true,
	}
};