use engine::game::rng::RNG;
use engine::game::{GameConfig, HoleData};
use engine::search::eval::Weights;

use crate::sim::run_batch;

pub const N_PARAMS: usize = 32;

pub struct SpsaConfig {
  pub a: f64,
  pub c: f64,
  pub alpha: f64,
  pub gamma: f64,
  pub big_a: f64,
  pub n_batch: usize,
  pub depth: u8,
  pub max_moves: u32,
  pub seed: u64,
  pub conv_window: usize,
  pub conv_threshold: f64,
}

impl Default for SpsaConfig {
  fn default() -> Self {
    SpsaConfig {
      a: 5.0,
      c: 0.1,
      alpha: 0.602,
      gamma: 0.101,
      big_a: 100.0,
      n_batch: 20,
      depth: 6,
      max_moves: 800,
      seed: 0x1337_CAFE,
      conv_window: 50,
      conv_threshold: 1e-5,
    }
  }
}

// flatten weights into a 32-element vec (non-tuned fields excluded)
pub fn weights_to_vec(w: &Weights) -> Vec<f64> {
  let mut v = Vec::with_capacity(N_PARAMS);
  v.push(w.outer_height);
  v.push(w.inner_height);
  v.push(w.unevenness);
  for &x in &w.wells {
    v.push(x);
  }
  v.push(w.holes.holes);
  v.push(w.holes.depth);
  v.push(w.holes.accessible);
  v.push(w.holes.inaccessible);
  for row in &w.clear {
    for &x in row {
      v.push(x);
    }
  }
  v.push(w.sent);
  v.push(w.surge);
  v.push(w.combo);
  v
}

// index layout:
//  0        outer_height
//  1        inner_height
//  2        unevenness
//  3..=12   wells[0..9]
//  13       holes.holes
//  14       holes.depth
//  15       holes.accessible
//  16       holes.inaccessible
//  17..=28  clear[0..2][0..3]
//  29       sent
//  30       surge
//  31       combo
pub fn vec_to_weights(v: &[f64]) -> Weights {
  let wells: [f64; 10] = std::array::from_fn(|i| v[3 + i]);
  let clear: [[f64; 4]; 3] =
    std::array::from_fn(|row| std::array::from_fn(|col| v[17 + row * 4 + col]));
  Weights {
    outer_height: v[0],
    inner_height: v[1],
    unevenness: v[2],
    wells,
    holes: HoleData {
      holes: v[13],
      depth: v[14],
      accessible: v[15],
      inaccessible: v[16],
    },
    clear,
    sent: v[29],
    surge: v[30],
    combo: v[31],
    t_hole: 0.0,
    i_hole: 0.0,
    waste: [0.0; 7],
  }
}

// per-param scale: max(|θ_i|, 1.0) — keeps small params tunable
pub fn build_scale(theta: &[f64]) -> Vec<f64> {
  theta.iter().map(|&x| x.abs().max(1.0)).collect()
}

// one SPSA step. returns (new_theta, win_rate_plus, avg_param_delta)
pub fn spsa_step(
  theta: &[f64],
  scale: &[f64],
  k: usize,
  cfg: &SpsaConfig,
  config: &GameConfig,
) -> (Vec<f64>, f64, f64) {
  let n = theta.len();

  let c_k = cfg.c / ((k + 1) as f64).powf(cfg.gamma);
  let a_k = cfg.a / (cfg.big_a + (k + 1) as f64).powf(cfg.alpha);

  // rademacher perturbation vector (+-1 per param)
  let mut rng = RNG::new(cfg.seed.wrapping_add(k as u64 * 6_364_136_223_846_793_005));
  let delta: Vec<f64> = (0..n)
    .map(|_| if rng.next() & 1 == 0 { 1.0 } else { -1.0 })
    .collect();

  let theta_plus: Vec<f64> = (0..n)
    .map(|i| theta[i] + c_k * delta[i] * scale[i])
    .collect();
  let theta_minus: Vec<f64> = (0..n)
    .map(|i| theta[i] - c_k * delta[i] * scale[i])
    .collect();

  let w_plus = vec_to_weights(&theta_plus);
  let w_minus = vec_to_weights(&theta_minus);

  let batch_seed = cfg.seed.wrapping_add(k as u64 * 1_000_000_007);
  let t_batch = std::time::Instant::now();
  let win_rate_plus = run_batch(
    &w_plus,
    &w_minus,
    cfg.n_batch,
    batch_seed,
    config,
    cfg.depth,
    cfg.max_moves,
  );
  println!(
    "  batch done: {:.2}s  wr={:.3}  ({} games, depth {})",
    t_batch.elapsed().as_secs_f64(),
    win_rate_plus,
    cfg.n_batch,
    cfg.depth,
  );

  let net = 2.0 * win_rate_plus - 1.0;

  let mut new_theta = theta.to_vec();
  let mut total_delta = 0.0f64;
  for i in 0..n {
    let grad_i = net * delta[i] / (2.0 * c_k * scale[i]);
    let step = a_k * grad_i * scale[i];
    new_theta[i] += step;
    total_delta += step.abs();
  }
  let avg_delta = total_delta / n as f64;

  (new_theta, win_rate_plus, avg_delta)
}
