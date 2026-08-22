use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use chrono::Local;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, reload};
use types::{Config, LoggingConfig};

static FILTER_HANDLE: OnceLock<reload::Handle<EnvFilter, tracing_subscriber::Registry>> =
  OnceLock::new();

pub struct LineRotatingFile {
  base_path: PathBuf,
  file: File,
  line_count: usize,
  logging_config: LoggingConfig
}

impl LineRotatingFile {
  pub fn new(base_path: PathBuf, config: LoggingConfig) -> io::Result<Self> {
    if let Some(parent) = base_path.parent() {
      fs::create_dir_all(parent)?;
    }

    let line_count = if base_path.exists() {
      let file = File::open(&base_path)?;
      BufReader::new(file).lines().count()
    } else {
      0
    };

    let file = OpenOptions::new().create(true).append(true).open(&base_path)?;

    println!("Logging base_path: {:#?}", base_path);

    Ok(Self { base_path, file, line_count, logging_config: config })
  }

  fn rotate_if_needed(&mut self, additional_lines: usize) -> io::Result<()> {
    if self.line_count + additional_lines < self.logging_config.max_lines_per_file {
      return Ok(());
    }

    self.file.flush()?;

    let file_name = self.base_path.file_name().expect("log file name missing").to_str().expect("Could not read the log file name");
    let date = Local::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    let new_file_name = self.base_path.with_file_name(format!("{file_name}.{date}"));

    fs::rename(file_name, new_file_name)?;

    // todo delete the oldest file

    self.file = OpenOptions::new().create(true).append(true).open(&self.base_path)?;
    self.line_count = 0;
    Ok(())
  }
}

impl Write for LineRotatingFile {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let newline_count = buf.iter().filter(|&&b| b == b'\n').count();
    self.rotate_if_needed(newline_count)?;
    let written = self.file.write(buf)?;
    self.line_count += newline_count;
    Ok(written)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.file.flush()
  }
}

fn build_filter(log_level: &str) -> EnvFilter {
  EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level))
}

pub fn init_tracing(config: &Config) -> WorkerGuard {
  let time_fmt = String::from("%Y-%m-%d %H:%M:%S%.6f");
  let (filter_layer, reload_handle) = reload::Layer::new(build_filter(&config.logging_config.log_level));
  let stdout_layer = fmt::layer()
    .with_writer(std::io::stdout)
    .with_timer(fmt::time::ChronoLocal::new(time_fmt.clone()))
    // .with_thread_ids(true)
    .with_thread_names(true)
    // .with_file(true)
    .with_line_number(true)
    .with_ansi(true);

  let file_appender = LineRotatingFile::new(PathBuf::from("logs/mods.log"), config.logging_config.clone())
    .expect("failed to initialize rotating log file");
  let (non_blocking, guard) = non_blocking(file_appender);
  let file_layer = fmt::layer()
    .with_writer(non_blocking)
    .with_timer(fmt::time::ChronoLocal::new(time_fmt))
    .with_thread_ids(true)
    .with_thread_names(true)
    // .with_file(true)
    .with_line_number(true)
    .with_ansi(false);

  tracing_subscriber::registry().with(filter_layer).with(stdout_layer).with(file_layer).init();

  let _ = FILTER_HANDLE.set(reload_handle);
  guard
}

pub fn set_log_level(log_level: &str) {
  if let Some(handle) = FILTER_HANDLE.get() {
    let _ = handle.reload(build_filter(log_level));
  }
}

#[cfg(test)]
mod tests {
  use super::build_filter;

  #[test]
  fn build_filter_uses_requested_level() {
    let filter = build_filter("debug");
    assert!(filter.to_string().contains("debug"));
  }
}
