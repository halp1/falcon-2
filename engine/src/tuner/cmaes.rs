use super::sim::run_solo;
use cmaes::{CMAESOptions, DVector, Mode};
use engine::{
  game::GameConfig,
  search::eval::{WEIGHTS_HANDTUNED, Weights},
};
use std::sync::{
  Arc,
  atomic::{AtomicU64, Ordering},
};

fn to_weights(x: &DVector<f64>) -> Weights {
  x.iter().copied().collect::<Vec<f64>>().into()
}

pub fn tune<const DEPTH: u8, const WIDTH: usize>(
  config: GameConfig,
  moves: usize,
  garbage_frequency: usize,
  samples: usize,
  max_generations: usize,
  start_iter: usize,
  initial: Option<Weights>,
) {
  let initial_vec: Vec<f64> = initial.unwrap_or(WEIGHTS_HANDTUNED).into();

  let gen_seed = Arc::new(AtomicU64::new(0));
  let gen_seed_obj = Arc::clone(&gen_seed);
  let objective = move |x: &DVector<f64>| {
    let weights = to_weights(x);
    let base = gen_seed_obj.load(Ordering::Acquire);
    (0..samples)
      .map(|s| {
        run_solo::<DEPTH, WIDTH>(
          &weights,
          &config,
          moves,
          garbage_frequency,
          base.wrapping_add(s as u64),
        )
      })
      .sum::<f64>()
      / samples as f64
  };

  let mut state = CMAESOptions::new(initial_vec, 1.0)
    .mode(Mode::Maximize)
    .enable_printing(1)
    .max_generations(max_generations.saturating_sub(start_iter))
    .build(objective)
    .unwrap();

  let start = std::time::Instant::now();
  loop {
    gen_seed.store(rand::random::<u64>(), Ordering::Release);
    match state.next_parallel() {
      None => {
        if let Some(best) = state.overall_best_individual() {
          let w = to_weights(&best.point);
          if let Ok(json) = serde_json::to_string_pretty(&w) {
            let _ = std::fs::write("tuning/weights_checkpoint.json", json);
          } else {
            eprintln!("Failed to serialize weights");
          }
        }
      }
      Some(termination) => {
        state.print_final_info(&termination.reasons);
        if let Some(best) = termination.overall_best {
          let w = to_weights(&best.point);
          let json = serde_json::to_string_pretty(&w).unwrap();
          std::fs::write("tuning/weights_best.json", &json).unwrap();
          println!("{json}");
        }
        break;
      }
    }
  }

  println!("Total time: {}s", start.elapsed().as_secs_f64());
}
