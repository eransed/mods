use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, MissedTickBehavior};
use tracing::info;

use crate::openprotocol::config::Config;
use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};
use crate::openprotocol::mid_0001::{Mid0001, Mid0001Rev7};
use crate::openprotocol::mid_0003::Mid0003;
use crate::openprotocol::mid_0005::mid_parse_0005;
use crate::openprotocol::mid_9999::Mid9999;

pub const TEST_ADDRESS: &str = "192.168.0.47:4545";

pub struct Client {
  config: Config,
  stream: TcpStream,
  receive_buffer: Vec<u8>,
}

impl Client {
  pub async fn connect(config: Config, address: &str) -> io::Result<Self> {
    let stream = TcpStream::connect(address).await?;
    Ok(Self { config, stream, receive_buffer: Vec::new() })
  }

  pub async fn run(mut self) -> io::Result<()> {
    self.send(&mid_0001(&self.config)).await?;

    let mut keep_alive = time::interval(Duration::from_secs(10));
    keep_alive.set_missed_tick_behavior(MissedTickBehavior::Delay);
    keep_alive.tick().await;

    loop {
      let mut read_buffer = [0_u8; 4096];
      tokio::select! {
        result = self.stream.read(&mut read_buffer) => {
          let bytes_read = result?;
          if bytes_read == 0 {
            return Ok(());
          }
          self.receive_buffer.extend_from_slice(&read_buffer[..bytes_read]);
          while let Some(message) = take_message(&mut self.receive_buffer)? {
            self.handle_received(&message);
          }
        }
        _ = keep_alive.tick() => {
          self.send(&mid_9999()).await?;
        }
        result = tokio::signal::ctrl_c() => {
          result?;
          self.send(&mid_0003()).await?;
          self.wait_for_stop_ack().await?;
          return Ok(());
        }
      }
    }
  }

  async fn wait_for_stop_ack(&mut self) -> io::Result<()> {
    loop {
      let mut read_buffer = [0_u8; 4096];
      let bytes_read = self.stream.read(&mut read_buffer).await?;
      if bytes_read == 0 {
        return Ok(());
      }
      self.receive_buffer.extend_from_slice(&read_buffer[..bytes_read]);
      while let Some(message) = take_message(&mut self.receive_buffer)? {
        let is_stop_ack = mid_parse_0005(&message).map(|ack| ack.mid_number == 3).unwrap_or(false);
        self.handle_received(&message);
        if is_stop_ack {
          return Ok(());
        }
      }
    }
  }

  async fn send<M: Mid>(&mut self, message: &M) -> io::Result<()> {
    let serialized = message.str();
    info!("SEND: {}", serialized);
    self.stream.write_all(serialized.as_bytes()).await?;
    self.stream.write_all(&[0]).await
  }

  fn handle_received(&self, message: &str) {
    match mid_parse_header(message) {
      Ok(header) => {
        info!("RECV: {}", message);
        match header.mid {
          5 => match mid_parse_0005(message) {
            Ok(ack) => info!("RECV parsed MID 0005 acknowledging MID {:04}", ack.mid_number),
            Err(error) => info!("RECV parse error: {}", error),
          },
          _ => info!("RECV parsed MID {:04} revision {}", header.mid, header.rev),
        }
      }
      Err(error) => info!("RECV parse error: {}", error),
    }
  }
}

pub async fn client(config: &Config) -> io::Result<()> {
  let config = Config { mid_0001_revision: config.mid_0001_revision };
  Client::connect(config, TEST_ADDRESS).await?.run().await
}

fn mid_0001(config: &Config) -> Mid0001 {
  let revision = config.mid_0001_revision as u16;
  Mid0001 {
    header: MidHeader {
      len: if revision == 7 { 23 } else { 20 },
      mid: 1,
      rev: revision,
      ..Default::default()
    },
    rev7: Mid0001Rev7 { optional_keep_alive: false },
  }
}

fn mid_0003() -> Mid0003 {
  Mid0003 { header: MidHeader { len: 20, mid: 3, rev: 1, ..Default::default() } }
}

fn mid_9999() -> Mid9999 {
  Mid9999 {
    header: MidHeader { len: 20, mid: 9999, rev: 1, ..Default::default() },
    data: String::new(),
  }
}

fn take_message(buffer: &mut Vec<u8>) -> io::Result<Option<String>> {
  if buffer.len() < 20 {
    return Ok(None);
  }

  let header = std::str::from_utf8(&buffer[..20])
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
  let length = header[..4]
    .parse::<usize>()
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
  if length < 20 {
    return Err(io::Error::new(
      io::ErrorKind::InvalidData,
      "MID length is shorter than its header",
    ));
  }
  if buffer.len() < length {
    return Ok(None);
  }

  let message = std::str::from_utf8(&buffer[..length])
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
    .to_owned();
  buffer.drain(..length);
  if buffer.first() == Some(&0) {
    buffer.remove(0);
  }
  Ok(Some(message))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_split_message() {
    let message = "00200003001000000000";
    let mut buffer = message.as_bytes()[..7].to_vec();
    assert!(take_message(&mut buffer).unwrap().is_none());
    buffer.extend_from_slice(&message.as_bytes()[7..]);
    assert_eq!(take_message(&mut buffer).unwrap().as_deref(), Some(message));
  }

  #[test]
  fn extracts_multiple_messages_and_nul_terminators() {
    let first = "00200003001000000000";
    let second = "00200099001000000000";
    let mut buffer = format!("{}\0{}\0", first, second).into_bytes();
    assert_eq!(take_message(&mut buffer).unwrap().as_deref(), Some(first));
    assert_eq!(take_message(&mut buffer).unwrap().as_deref(), Some(second));
    assert!(buffer.is_empty());
  }
}
