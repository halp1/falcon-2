use super::sim::run_solo;
use cmaes::{CMAESOptions, DVector, Mode};
use engine::{
  game::GameConfig,
  search::eval::{WEIGHTS_HANDTUNED, Weights},
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
) {
  let initial: Vec<f64> = WEIGHTS_HANDTUNED.into();

  let objective = move |x: &DVector<f64>| {
    let weights = to_weights(x);
    (0..samples)
      .map(|_| run_solo::<DEPTH, WIDTH>(&weights, &config, moves, garbage_frequency))
      .sum::<f64>()
      / samples as f64
  };

  let mut state = CMAESOptions::new(initial, 1.0)
    .mode(Mode::Maximize)
    .enable_printing(1)
    .max_generations(max_generations)
    .build(objective)
    .unwrap();

  let start = std::time::Instant::now();
  loop {
    match state.next_parallel() {
      None => {
        if let Some(best) = state.overall_best_individual() {
          let w = to_weights(&best.point);
          if let Ok(json) = serde_json::to_string_pretty(&w) {
            let _ = std::fs::write("weights/weights_checkpoint.json", json);
          }
        }
      }
      Some(termination) => {
        state.print_final_info(&termination.reasons);
        if let Some(best) = termination.overall_best {
          let w = to_weights(&best.point);
          let json = serde_json::to_string_pretty(&w).unwrap();
          std::fs::write("weights/weights_best.json", &json).unwrap();
          println!("{json}");
        }
        break;
      }
    }
  }

  println!("Total time: {}s", start.elapsed().as_secs_f64());
}
