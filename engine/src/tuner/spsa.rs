use super::sim::batch_match;
use engine::{
  game::GameConfig,
  search::eval::{WEIGHTS_HANDTUNED, Weights},
};

pub fn tune<const DEPTH: u8, const WIDTH: usize>(
  config: GameConfig,
  max_moves: usize,
  games: usize,
  steps: usize,
  a: f64,
  c: f64,
  start_iter: usize,
  initial: Option<Weights>,
) {
  const ALPHA: f64 = 0.602;
  const GAMMA: f64 = 0.101;
  const EVAL_EVERY: usize = 10;
  const EVAL_GAMES: usize = 20;
  let big_a = steps as f64 * 0.1;

  let reference: Weights = WEIGHTS_HANDTUNED;
  let mut theta: Vec<f64> = initial.unwrap_or(WEIGHTS_HANDTUNED).into();
  let n = theta.len();

  let start = std::time::Instant::now();
  for k in start_iter..steps {
    let a_k = a / (k as f64 + big_a + 1.0).powf(ALPHA);
    let c_k = c / (k as f64 + 1.0).powf(GAMMA);

    let delta: Vec<f64> = (0..n)
      .map(|_| if rand::random::<bool>() { 1.0 } else { -1.0 })
      .collect();

    let theta_plus: Weights = theta
      .iter()
      .zip(&delta)
      .map(|(t, d)| t + c_k * d)
      .collect::<Vec<f64>>()
      .into();
    let theta_minus: Weights = theta
      .iter()
      .zip(&delta)
      .map(|(t, d)| t - c_k * d)
      .collect::<Vec<f64>>()
      .into();

    let seed = rand::random::<u64>();
    let win_rate =
      batch_match::<DEPTH, WIDTH>(&theta_plus, &theta_minus, games, &config, max_moves, seed);

    for i in 0..n {
      theta[i] += a_k * (win_rate - 0.5) / (c_k * delta[i]);
    }

    // if k % 10 == 0 || k == steps - 1 {
    let elapsed = start.elapsed().as_secs_f64();
    let w: Weights = theta.clone().into();
    if let Ok(json) = serde_json::to_string_pretty(&w) {
      let _ = std::fs::write("tuning/weights_checkpoint.json", json);
    }
    print!("step={k:4} win={win_rate:.3} a_k={a_k:.4} c_k={c_k:.4} t={elapsed:.1}s");
    if k % EVAL_EVERY == 0 || k == steps - 1 {
      let vs_ref = batch_match::<DEPTH, WIDTH>(
        &w,
        &reference,
        EVAL_GAMES,
        &config,
        max_moves,
        rand::random::<u64>(),
      );
      print!("  vs_ref={vs_ref:.3}");
    }
    println!();
    // }
  }

  let final_weights: Weights = theta.into();
  let json = serde_json::to_string_pretty(&final_weights).unwrap();
  std::fs::write("tuning/weights_best.json", &json).unwrap();
  println!("{json}");
  println!("Total time: {}s", start.elapsed().as_secs_f64());
}
