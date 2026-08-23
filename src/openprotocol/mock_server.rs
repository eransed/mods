use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};
use crate::openprotocol::mid_9999::Mid9999;

pub const MOCK_SERVER_PORT: u16 = 5555;

pub async fn run(mut shutdown_rx: watch::Receiver<bool>) -> io::Result<()> {
  let listener = TcpListener::bind(("127.0.0.1", MOCK_SERVER_PORT)).await?;
  info!(port = MOCK_SERVER_PORT, "OpenProtocol mock server listening");
  let mut connections = JoinSet::new();

  loop {
    tokio::select! {
      Some(result) = connections.join_next(), if !connections.is_empty() => {
        if let Err(error) = result {
          warn!(%error, "OpenProtocol mock connection task stopped unexpectedly");
        }
      }
      result = listener.accept() => {
        match result {
          Ok((stream, peer)) => {
            connections.spawn(handle_connection(stream));
            info!(%peer, "OpenProtocol mock client connected");
          }
          Err(error) => warn!(%error, "OpenProtocol mock accept failed"),
        }
      }
      result = shutdown_rx.changed() => {
        if result.is_err() || *shutdown_rx.borrow() {
          connections.abort_all();
          while connections.join_next().await.is_some() {}
          return Ok(());
        }
      }
    }
  }
}

async fn handle_connection(mut stream: TcpStream) {
  let mut buffer = Vec::new();
  let mut read_buffer = [0_u8; 4096];

  loop {
    let bytes_read = match stream.read(&mut read_buffer).await {
      Ok(0) => return,
      Ok(bytes_read) => bytes_read,
      Err(error) => {
        warn!(%error, "OpenProtocol mock read failed");
        return;
      }
    };
    buffer.extend_from_slice(&read_buffer[..bytes_read]);

    while let Some(separator) = buffer.iter().position(|byte| *byte == 0) {
      let message = buffer.drain(..separator).collect::<Vec<_>>();
      buffer.drain(..1);
      let message = match String::from_utf8(message) {
        Ok(message) => message,
        Err(error) => {
          warn!(%error, "OpenProtocol mock received invalid UTF-8");
          continue;
        }
      };

      if matches!(mid_parse_header(&message), Ok(header) if header.mid == 9999)
        && let Err(error) = send_keep_alive_response(&mut stream).await
      {
        error!(%error, "OpenProtocol mock response failed");
        return;
      }
    }
  }
}

async fn send_keep_alive_response(stream: &mut TcpStream) -> io::Result<()> {
  let response = Mid9999 {
    header: MidHeader { len: 20, mid: 9999, rev: 1, ..Default::default() },
    data: String::new(),
  };
  stream.write_all(response.str().as_bytes()).await?;
  stream.write_all(&[0]).await
}
