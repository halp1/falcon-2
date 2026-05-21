use std::{
  fs::{self, File},
  io::{self, Write},
  sync::Mutex,
  time::{Instant, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

pub struct WSLogger {
  file: Mutex<File>,
  initial_time: Instant,
}

impl WSLogger {
  pub fn new() -> io::Result<Self> {
    fs::create_dir_all("logs/ws")?;
    let ts = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis();
    let file = File::create(format!("logs/ws/{}.log", ts))?;
    Ok(Self {
      file: Mutex::new(file),
      initial_time: Instant::now(),
    })
  }

  fn ms_to_string(&self, ms: u128) -> String {
    format!(
      "{:02}:{:02}:{:02}.{:03}",
      ms / 3_600_000,
      (ms % 3_600_000) / 60_000,
      (ms % 60_000) / 1000,
      ms % 1000,
    )
  }

  pub fn push(&self, event_type: &str, command: &str, data: &impl Serialize) {
    let ms = self.initial_time.elapsed().as_millis();
    let time_str = self.ms_to_string(ms);
    let padded_type = format!("{:>width$}", event_type, width = "receive".len());
    let json_val = serde_json::json!({ "command": command, "data": data });
    let json_str = serde_json::to_string_pretty(&json_val).unwrap_or_default();
    let collapsed = {
      let with_spaces = json_str.replace('\n', " ");
      let mut result = with_spaces;
      while result.contains("  ") {
        result = result.replace("  ", " ");
      }
      result
    };
    let line = format!("[{} {}] {}\n", time_str, padded_type, collapsed);
    if let Ok(mut file) = self.file.lock() {
      file.write_all(line.as_bytes()).ok();
    }
  }
}
