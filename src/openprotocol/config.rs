
#[derive(Clone, Debug)]
pub struct MidConfig {
  pub rev: u16,
  pub active: bool,
}

#[derive(Clone, Debug)]
pub struct Config {
  pub ip: String,
  pub port: u16,
  pub keep_alive_time_ms: u64,
  pub reconnect_delay_ms: u64,
  pub mid_0001_config: MidConfig,
}
