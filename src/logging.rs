use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub struct LineRotatingFile {
    base_path: PathBuf,
    max_lines: usize,
    max_files: usize,
    file: File,
    line_count: usize,
}

impl LineRotatingFile {
    pub fn new(base_path: PathBuf, max_lines: usize, max_files: usize) -> io::Result<Self> {
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
        Ok(Self {
            base_path,
            max_lines,
            max_files,
            file,
            line_count,
        })
    }

    fn rotated_path(&self, index: usize) -> PathBuf {
        let file_name = self
            .base_path
            .file_name()
            .expect("log file name missing")
            .to_string_lossy();
        self.base_path.with_file_name(format!("{file_name}.{index}"))
    }

    fn rotate_if_needed(&mut self, additional_lines: usize) -> io::Result<()> {
        if self.line_count + additional_lines < self.max_lines {
            return Ok(());
        }

        self.file.flush()?;

        if self.max_files > 0 {
            let oldest = self.rotated_path(self.max_files);
            if oldest.exists() {
                fs::remove_file(&oldest)?;
            }

            for i in (1..self.max_files).rev() {
                let from = self.rotated_path(i);
                let to = self.rotated_path(i + 1);
                if from.exists() {
                    fs::rename(from, to)?;
                }
            }
        }

        if self.base_path.exists() {
            let rotated = self.rotated_path(1);
            fs::rename(&self.base_path, rotated)?;
        }

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

pub fn init_tracing() -> WorkerGuard {
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_timer(fmt::time::LocalTime::rfc_3339())
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    let file_appender = LineRotatingFile::new(PathBuf::from("logs/mods.log"), 20_000, 50)
        .expect("failed to initialize rotating log file");
    let (non_blocking, guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_timer(fmt::time::LocalTime::rfc_3339())
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}
