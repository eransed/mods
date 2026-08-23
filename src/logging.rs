use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::info;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, reload};
use types::{Config, LoggingConfig};

static FILTER_HANDLE: OnceLock<reload::Handle<EnvFilter, tracing_subscriber::Registry>> =
  OnceLock::new();

pub struct LineRotatingFile {
  base_path: PathBuf,
  file: File,
  line_count: usize,
  logging_config: LoggingConfig,
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

    info!("Logging base_path: {:#?}", base_path);

    Ok(Self { base_path, file, line_count, logging_config: config })
  }

  fn rotate_if_needed(&mut self, additional_lines: usize) -> io::Result<()> {
    // Compare log usage with the configured scalar limit before rotating.
    if self.line_count + additional_lines < self.logging_config.max_lines_per_file.value {
      // println!(
      //   "Current line count: {}, additional lines: {}, max lines per file: {}. No rotation needed.",
      //   self.line_count, additional_lines, self.logging_config.max_lines_per_file
      // );
      return Ok(());
    }

    self.file.flush()?;

    let file_name = self
      .base_path
      .file_name()
      .expect("log file name missing")
      .to_str()
      .expect("Could not read the log file name");
    let file_stem = self
      .base_path
      .file_stem()
      .expect("log file stem missing")
      .to_str()
      .expect("Could not read the log file stem");
    let file_extension = self.base_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let date = Local::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    let new_file_name =
      self.base_path.with_file_name(format!("{file_stem}.{date}.{file_extension}"));

    fs::rename(&self.base_path, &new_file_name).expect("Failed to rename log file");

    let log_files = fs::read_dir(self.base_path.parent().expect("log file parent missing"))?
      .filter_map(|entry| entry.ok())
      .filter(|entry| {
        entry.file_name().to_str().map_or(false, |name| {
          name == file_name
            || name.starts_with(&format!("{file_stem}."))
              && name.ends_with(&format!(".{file_extension}"))
        })
      })
      .collect::<Vec<_>>();

    let mut archived_files =
      log_files.into_iter().filter(|entry| entry.file_name() != file_name).collect::<Vec<_>>();
    archived_files.sort_by_key(|entry| entry.file_name());
    let files_to_remove = archived_files
      .len()
      // Keep the configured number of archived files after rotation.
      .saturating_sub(self.logging_config.max_log_file_to_keep.value.saturating_sub(1));
    for oldest_file in archived_files.into_iter().take(files_to_remove) {
      fs::remove_file(oldest_file.path())?;
      // println!("Deleted oldest log file: {}", oldest_file.path().display());
    }

    self.file = OpenOptions::new().create(true).append(true).open(&self.base_path)?;

    // Log the rotation event using, making sure to use the standard logging format:
    // println!("Rotated log file: {} -> {}", self.base_path.display(), &new_file_name.display());

    self.line_count = 1;
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
  // Build the logging filter from the configured log-level value.
  let (filter_layer, reload_handle) =
    reload::Layer::new(build_filter(&config.logging_config.log_level.value));
  let stdout_layer = fmt::layer()
    .with_writer(std::io::stdout)
    .with_timer(fmt::time::ChronoLocal::new(time_fmt.clone()))
    // .with_thread_ids(true)
    .with_thread_names(true)
    // .with_file(true)
    .with_line_number(true)
    .with_ansi(true);

  let file_appender =
    LineRotatingFile::new(PathBuf::from("logs/mods.log"), config.logging_config.clone())
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
  use super::{LineRotatingFile, build_filter};
  use std::fs;
  use std::io::Write;
  use std::path::PathBuf;
  use std::time::{SystemTime, UNIX_EPOCH};
  use types::LoggingConfig;

  fn test_log_path() -> PathBuf {
    let nanos =
      SystemTime::now().duration_since(UNIX_EPOCH).expect("system time should be valid").as_nanos();
    std::env::temp_dir().join(format!("mods-logging-test-{nanos}")).join("mods.log")
  }

  fn test_logging_config(max_log_file_to_keep: usize) -> LoggingConfig {
    // Override only the scalar values under test and retain property metadata.
    let mut config = LoggingConfig::default();
    config.max_lines_per_file.value = 1;
    config.max_log_file_to_keep.value = max_log_file_to_keep;
    config
  }

  fn create_log_file(path: &PathBuf, contents: &str) {
    fs::write(path, contents).expect("test log file should be created");
  }

  #[test]
  fn build_filter_uses_requested_level() {
    let filter = build_filter("debug");
    assert!(filter.to_string().contains("debug"));
  }

  #[test]
  fn rotation_does_not_exceed_file_limit() {
    let base_path = test_log_path();
    let parent = base_path.parent().expect("test log parent should exist");
    fs::create_dir_all(parent).expect("test log directory should be created");

    create_log_file(&base_path, "current\n");
    for index in 1..=2 {
      create_log_file(&parent.join(format!("mods.20260101_00000{index}.000.log")), "archived\n");
    }

    let mut log = LineRotatingFile::new(base_path.clone(), test_logging_config(3))
      .expect("rotating log should open");
    log.write_all(b"rotated\n").expect("rotation should succeed");
    log.flush().expect("log should flush");

    let file_count = fs::read_dir(parent).expect("test log directory should be readable").count();
    assert_eq!(file_count, 3);
    let _ = fs::remove_dir_all(parent);
  }

  #[test]
  fn rotation_respects_max_lines_per_file() {
    let base_path = test_log_path();
    let parent = base_path.parent().expect("test log parent should exist");
    fs::create_dir_all(parent).expect("test log directory should be created");

    create_log_file(&base_path, "first\nsecond\n");
    let mut config = test_logging_config(3);
    // Lower the line threshold to force a rotation in this test.
    config.max_lines_per_file.value = 2;
    let mut log =
      LineRotatingFile::new(base_path.clone(), config).expect("rotating log should open");
    log.write_all(b"third\n").expect("rotation should succeed");
    log.flush().expect("log should flush");

    for entry in fs::read_dir(parent).expect("test log directory should be readable") {
      let path = entry.expect("test log entry should be readable").path();
      let line_count =
        fs::read_to_string(path).expect("test log file should be readable").lines().count();
      assert!(line_count <= 2, "log file contains {line_count} lines");
    }
    let _ = fs::remove_dir_all(parent);
  }

  #[test]
  fn rotation_deletes_oldest_log_file() {
    let base_path = test_log_path();
    let parent = base_path.parent().expect("test log parent should exist");
    fs::create_dir_all(parent).expect("test log directory should be created");

    create_log_file(&base_path, "current\n");
    create_log_file(&parent.join("mods.20200101_000000.000.log"), "oldest\n");
    create_log_file(&parent.join("mods.20260101_000000.000.log"), "newer\n");
    create_log_file(&parent.join("mods.20260101_000001.000.log"), "newest\n");

    let mut log = LineRotatingFile::new(base_path.clone(), test_logging_config(3))
      .expect("rotating log should open");
    log.write_all(b"rotated\n").expect("rotation should succeed");

    assert!(!parent.join("mods.20200101_000000.000.log").exists());
    assert!(parent.join("mods.20260101_000001.000.log").exists());
    assert_eq!(fs::read_dir(parent).expect("test log directory should be readable").count(), 3);
    let _ = fs::remove_dir_all(parent);
  }
}
