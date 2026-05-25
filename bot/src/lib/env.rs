use clap::Parser;
use std::sync::OnceLock;
use validator::Validate;

#[derive(Parser, Validate, Debug)]
pub struct Env {
  #[arg(long, env = "TOKEN")]
  #[validate(length(min = 1, message = "Token cannot be empty"))]
  pub token: String,

  #[arg(long, default_value_t = false)]
  pub server: bool,

  #[arg(long, env = "WEIGHTS", default_value_t = String::from("weights/weights.json"))]
  pub weights: String,
}

static ENV: OnceLock<Env> = OnceLock::new();

pub fn env() -> &'static Env {
  ENV.get().expect("Env must be initialized before access")
}

pub fn parse_env() {
  let parsed_env = Env::parse();
  if let Err(errors) = parsed_env.validate() {
    eprintln!("Envuration error:\n{}", errors);
    std::process::exit(1);
  }

  ENV.set(parsed_env).unwrap();
}
