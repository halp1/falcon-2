mod sim;
mod spsa;

use std::io::Write;

use engine::game::GameConfig;
use engine::search::eval::{WEIGHTS_HANDTUNED, Weights};
use spsa::{SpsaConfig, build_scale, spsa_step, vec_to_weights, weights_to_vec};
use triangle::engine::utils::KickTable;
use triangle::types::game::{ComboTable, SpinBonuses};

fn default_config() -> GameConfig {
  GameConfig {
    kicks: KickTable::SRSPlus,
    spins: SpinBonuses::AllMiniPlus,
    b2b_chaining: true,
    b2b_charging: true,
    b2b_charge_at: 0,
    b2b_charge_base: 0,
    pc_b2b: 1,
    pc_send: 5,
    combo_table: ComboTable::Multiplier,
    garbage_multiplier: 1.0,
    garbage_cap: 8,
    garbage_special_bonus: true,
  }
}

fn write_weights(path: &str, w: &Weights) {
  match serde_json::to_string_pretty(w) {
    Ok(json) => {
      if let Err(e) = std::fs::write(path, json) {
        eprintln!("write {}: {}", path, e);
      }
    }
    Err(e) => eprintln!("serialize {}: {}", path, e),
  }
}

fn print_key_weights(w: &Weights) {
  println!(
    "  outer={:.2} inner={:.2} uneven={:.2} sent={:.2} combo={:.2} surge={:.2}",
    w.outer_height, w.inner_height, w.unevenness, w.sent, w.combo, w.surge
  );
  println!(
    "  wells: {:?}",
    w.wells
      .iter()
      .map(|x| format!("{:.1}", x))
      .collect::<Vec<_>>()
      .join(", ")
  );
  println!(
    "  clear[spin][lines]: none={:?} mini={:?} full={:?}",
    w.clear[0].map(|x| format!("{:.1}", x)),
    w.clear[1].map(|x| format!("{:.1}", x)),
    w.clear[2].map(|x| format!("{:.1}", x)),
  );
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let mut max_iter = 1000usize;
  let mut batch = 20usize;
  let mut depth = 6u8;
  let mut save_every = 50usize;
  let mut input_path: Option<String> = None;
  let mut output_path = String::from("tuning/weights_tuned.json");

  let mut i = 1;
  while i < args.len() {
    match args[i].as_str() {
      "--iter" => {
        max_iter = args[i + 1].parse().unwrap_or(max_iter);
        i += 2;
      }
      "--batch" => {
        batch = args[i + 1].parse().unwrap_or(batch);
        i += 2;
      }
      "--depth" => {
        depth = args[i + 1].parse().unwrap_or(depth);
        i += 2;
      }
      "--save-every" => {
        save_every = args[i + 1].parse().unwrap_or(save_every);
        i += 2;
      }
      "--input" => {
        input_path = Some(args[i + 1].clone());
        i += 2;
      }
      "--output" => {
        output_path = args[i + 1].clone();
        i += 2;
      }
      _ => i += 1,
    }
  }

  let initial_weights: Weights = input_path
    .as_ref()
    .and_then(|p| std::fs::read_to_string(p).ok())
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_else(|| WEIGHTS_HANDTUNED.clone());

  println!(
    "spsa tuner | iter={} batch={} depth={} save_every={}",
    max_iter, batch, depth, save_every
  );
  println!("initial weights:");
  print_key_weights(&initial_weights);
  std::io::stdout().flush().ok();

  std::fs::create_dir_all("tuning").ok();

  let config = default_config();
  let cfg = SpsaConfig {
    n_batch: batch,
    depth,
    big_a: (max_iter as f64 * 0.1).max(100.0),
    ..SpsaConfig::default()
  };

  let scale = build_scale(&weights_to_vec(&initial_weights));
  let mut theta = weights_to_vec(&initial_weights);

  let mut best_win_rate = 0.0f64;
  let mut best_theta = theta.clone();

  let mut wr_window: std::collections::VecDeque<f64> =
    std::collections::VecDeque::with_capacity(cfg.conv_window + 1);
  let mut delta_window: std::collections::VecDeque<f64> =
    std::collections::VecDeque::with_capacity(cfg.conv_window + 1);

  let start = std::time::Instant::now();

  for k in 0..max_iter {
    let iter_start = std::time::Instant::now();
    print!("[{}/{}] running batches...", k + 1, max_iter);
    std::io::stdout().flush().ok();

    let (new_theta, win_rate, avg_delta) = spsa_step(&theta, &scale, k, &cfg, &config);
    let iter_secs = iter_start.elapsed().as_secs_f64();
    theta = new_theta;

    wr_window.push_back(win_rate);
    delta_window.push_back(avg_delta);
    if wr_window.len() > cfg.conv_window {
      wr_window.pop_front();
      delta_window.pop_front();
    }

    // track best weights by rolling win-rate
    if wr_window.len() >= 10 {
      let recent_avg = wr_window.iter().rev().take(10).sum::<f64>() / 10.0;
      if recent_avg > best_win_rate {
        best_win_rate = recent_avg;
        best_theta = theta.clone();
        write_weights("tuning/weights_best.json", &vec_to_weights(&best_theta));
      }
    }

    {
      let avg_wr = wr_window.iter().sum::<f64>() / wr_window.len() as f64;
      let avg_d = delta_window.iter().sum::<f64>() / delta_window.len() as f64;
      let elapsed = start.elapsed().as_secs_f64();
      let iters_per_sec = (k + 1) as f64 / elapsed;
      let eta_secs = (max_iter - k - 1) as f64 / iters_per_sec;
      println!(" done ({:.1}s)", iter_secs,);
      println!(
        "  wr={:.3} avg_wr={:.3} Δ={:.5} | {:.3} iter/s  total={:.1}s  ~{:.0}s left",
        win_rate, avg_wr, avg_d, iters_per_sec, elapsed, eta_secs,
      );
      print_key_weights(&vec_to_weights(&theta));
      std::io::stdout().flush().ok();
    }

    if (k + 1) % save_every == 0 {
      let path = format!("tuning/weights_{:06}.json", k + 1);
      write_weights(&path, &vec_to_weights(&theta));
      println!("  → snapshot {}", path);
      std::io::stdout().flush().ok();
    }

    // convergence: average param change across last conv_window iters is tiny
    if delta_window.len() >= cfg.conv_window {
      let avg_d = delta_window.iter().sum::<f64>() / delta_window.len() as f64;
      if avg_d < cfg.conv_threshold {
        println!(
          "converged at iteration {} (avg_delta={:.6} < {})",
          k + 1,
          avg_d,
          cfg.conv_threshold
        );
        break;
      }
    }
  }

  let final_w = vec_to_weights(&theta);
  write_weights(&output_path, &final_w);
  println!("final weights saved → {}", output_path);

  let best_w = vec_to_weights(&best_theta);
  write_weights("tuning/weights_best.json", &best_w);
  println!("best weights saved  → tuning/weights_best.json");

  println!("\nfinal weights:");
  print_key_weights(&final_w);
}
