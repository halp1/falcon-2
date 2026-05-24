use criterion::{Criterion, criterion_group, criterion_main};
use engine::{
  game::{
    Game, GameConfig, StartState,
    queue::{Bag, Queue},
  },
  search::{beam_search, eval::WEIGHTS_HANDTUNED, movegen::expand},
};
use triangle::{
  engine::{queue::Mino, utils::KickTable},
  types::game::{ComboTable, Spin, SpinBonuses},
};

fn setup() -> (GameConfig, Game, Queue<32>) {
  let config = GameConfig {
    kicks: KickTable::SRSX,
    spins: SpinBonuses::Handheld,
    b2b_chaining: false,
    b2b_charging: true,
    b2b_charge_at: 0,
    b2b_charge_base: 0,
    pc_b2b: 1,
    pc_send: 5,
    combo_table: ComboTable::Multiplier,
    garbage_multiplier: 1.0,
    garbage_cap: 8,
    garbage_special_bonus: true,
  };

  let mut queue = Queue::new(Bag::Bag7, 0, vec![Mino::Z]);
  let game = Game::new(queue.shift());

  (config, game, queue)
}

fn bench_beam_search(c: &mut Criterion) {
  let (config, game, queue) = setup();
  let start_state = StartState {
    queue: &queue.as_array(),
    garbage: &[],
  };

  c.bench_function("beam_search d7/w1000", |b| {
    b.iter(|| beam_search::<7, 1000>(game.clone(), &config, &start_state, &WEIGHTS_HANDTUNED))
  });
}

fn bench_expand(c: &mut Criterion) {
  let (config, mut game, queue) = setup();
  let mut passed = [0u64; 2048];
  let mut res = [(0u8, 0u8, 0u8, Spin::None); 512];

  let map = game.collision_map();
  let start_state = StartState {
    queue: &queue.as_array(),
    garbage: &[],
  };

  c.bench_function("expand movegen", |b| {
    b.iter(|| {
      expand(
        &mut game,
        &config,
        &map,
        &start_state,
        &mut passed,
        &mut res,
      )
    })
  });
}

criterion_group!(benches, bench_beam_search, bench_expand);
criterion_main!(benches);
