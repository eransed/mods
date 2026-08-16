//! ToolAPI adapter
//!
//! Opens a TCP server compatible with the ILG ToolAPI
//!
//! Messages are JSON encoded UTF-8 strings ending with a NUL byte (\0).
//! Messages are sent and received in plain text, no encryption is currently implemented.
//!
//! Supported messages are defined in the [`ToolAPIMessage`] enum
//!
//! This implementation have support for segmented messages (messages that span multiple buffers) and
//! mulitpart messages (mutltiple messages in the same buffer).
//!
//! The order of the entries of objects ```{...}``` are arbitrary and spaces outside of "" are ignored, e.g. ```{"a" : 1, "b" : 2}``` is the same as ```{"b":2,"a":1}```.
//!
//! Just arrays ```[...]``` keep the order of elements, e.g. ```[1,2]``` is not equal to ```[2,1]```.
//!
//! For transfer size and parsing performance minimize the use of whitespace.
//!
//! Additional entries in messages, which are not specified here, shall not result in errors.
//!
//! This enables backwards compatibility and test of experimental features.
//!
//! All messages, which contain arrays, shall also work with empty arrays, probably doing nothing then.

use std::{net::SocketAddr, sync::mpsc::Sender};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast,
    // sync::broadcast,
};

const BUFFER_SIZE: usize = 8192;
const PORT: u16 = 8124;
const HOST: &str = "0.0.0.0";

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Station {
    description: String,
    id: String,
    location: String,
    name: String,
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "code")]
pub enum ErrorCode {
    /// Wrong parameters, no connection
    MQTT001 {},
    /// No connection
    MQTT002 {},
    /// No connection or error
    TMN001 {},
    /// No connection or error
    OTC001 {},
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "severity")]
pub enum Severity {
    /// Just for information, no consequences (e.g. adapters version is 1.2.3; a few(!) log outputs), will be put in ILG log
    info {},
    /// Shall be shown in the UI of ILG, but there is no immediate influence on the functionality (e.g. X will be deprecated in a future version; 100 positions at once will decrease the performance; there is only 10% space left on the device; ...)
    warning {},
    /// Something did not work (e.g. a message has an unknown or missing topic; wrong parameters in a message; ...)
    error {},
    /// The system is broken and needs fixing by hand (e.g. out of memory; something broken; ...)
    fatal {},
}

/// The variants in this enum represents the supported messages topics in the ToolAPI that the adapter can send to ILG.
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "topic")]
pub enum ToolAPIMessageAdapterSignal {
    /// Adapter response to a ping message.
    ///
    /// Example:
    /// ```not_rust
    /// {"topic": "pong"}
    /// ```
    #[serde(rename = "pong")]
    pong {},
}

/// The variants in this enum represents the supported messages topics in the ToolAPI that ILG will send to the connected adapter.
/// The JSON encoded messages have a internal tag ```topic``` that defines the specific message structure.
///
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "topic")]
pub enum ToolAPIMessage {
    /// Sent by both adapters and ILG.
    /// This can be used by adapters to indicate warnings and errors, but also to inform about stuff (may be used as a debug channel).
    error {
        /// Use the topic of the erroneous message or for other stuff use some useful names like "log" or "version".
        about: String,

        /// An english description of the error (preferred in a useful wording: X went wrong, because parameter Y is not in range, allowed range is: A-B); or the information in case of info.
        reason: String,

        /// A code from a list of possible errors.
        code: String,
        severity: String,
        persistent: bool,
    },
    /// ILG will send a ping message in a regualar intervall (750ms)
    /// Adapters may respond with a pong message.
    ///
    /// Example:
    /// ```not_rust
    /// {"topic": "ping"}
    /// ```
    #[serde(rename = "ping")]
    ping {},

    /// ILG will send this message once connected to a adapter.
    start {
        ilgVersion: String,
        timestamp: u64,
        station: Station,
    },

    /// Sent by ILG when the process has be reset.
    ilg_resetted {},
    sensor_ready {
        sensor: i32,
        status: bool,
    },
    workpiece_started {
        workpieceId: String,
        workplaceId: String,
    },
    workpiece_finished {
        status: String,
        completed: bool,
        workplaceId: String,
    },
    entered_position {
        positionId: String,
        positionNumber: i32,
        workpieceId: String,
        toolId: String,
        groupId: String,
        workplaceId: String,
    },
    left_position {
        positionId: String,
        positionNumber: i32,
        workpieceId: String,
        toolId: String,
        groupId: String,
        workplaceId: String,
    },

    /// ```not_rust
    /// {"topic": "lock_tool", "toolId": "default"}
    /// ```
    lock_tool {
        toolId: String,
    },

    /// ```not_rust
    /// {"topic": "release_tool", "toolId": "default"}
    /// ```
    release_tool {
        toolId: String,
    },
    current_position {
        positionId: String,
        positionNumber: i32,
        workpieceId: String,
        workplaceId: String,
        tools: Vec<String>,
    },
    next_position {
        workplaceId: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        positionNumber: Option<i32>,

        #[serde(skip_serializing_if = "Option::is_none")]
        positionId: Option<String>,
    },
}

impl ToolAPIMessage {
    fn json_str(&self) -> String {
        serde_json::to_string(self).expect("Failed to stringify message")
    }
}

impl ToolAPIMessageAdapterSignal {
    fn json_str(&self) -> String {
        serde_json::to_string(self).expect("Failed to stringify message")
    }
}

pub fn start(sender: Sender<String>, broadcast_channel: tokio::sync::broadcast::Sender<String>) {
    let threads = 4;
    info!("Creating a tokio runtime with {} threads", threads);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            adapter_server(sender, broadcast_channel).await;
        })
}

async fn handle_message(m: &str, socket: &mut TcpStream, addr: SocketAddr) {
    match serde_json::from_str::<ToolAPIMessage>(m) {
        Ok(v) => {
            match v {
                ToolAPIMessage::ping {} => (),
                ToolAPIMessage::error { .. } => (),
                _ => {
                    info!("Handle message: {:?}", v);
                }
            }

            match v {
                ToolAPIMessage::workpiece_started { .. } => {
                    let next_position = ToolAPIMessage::next_position {
                        workplaceId: String::from("workplace1"),
                        positionNumber: Some(5),
                        positionId: None,
                    };
                    let send_buf = next_position.json_str() + "\0";
                    if let Err(e) = socket.write_all(&send_buf.into_bytes()).await {
                        error!("Failed to write next_position to socket {}: {}", addr, e);
                        return;
                    }
                    info!("next_position has been sent");
                }
                ToolAPIMessage::error { about, reason, .. } => {
                    warn!("Error: {}: {}", about, reason);
                }
                ToolAPIMessage::ping {} => {
                    let pong = ToolAPIMessageAdapterSignal::pong {};
                    let send_buf = pong.json_str() + "\0";
                    if let Err(e) = socket.write_all(&send_buf.into_bytes()).await {
                        error!("Failed to write pong to socket {}: {}", addr, e);
                        return;
                    }
                }
                _ => (),
            }
        }
        Err(_) => {
            error!("Failed to parse message part of size {}: {}", m.len(), m);
        }
    }
}

async fn process_socket() {
    
}

async fn adapter_server(
    sender: Sender<String>,
    broadcast_channel: tokio::sync::broadcast::Sender<String>,
) {
    let server_addr = format!("{}:{}", HOST, PORT);
    let listener = tokio::net::TcpListener::bind(&server_addr)
        .await
        .expect(format!("Failed to start tcp server on {}", &server_addr).as_str());

    info!("adapter started on address {}", server_addr);

    // let (tx, _) = broadcast::channel::<Message>(16);

    // let message = Message::default();

    // let _ = tx.send(message).expect("Broadcast failed");

    // warn!("TODO broadcast modules wip");

    loop {
        debug!("Waiting for new connections...");
        let (mut socket, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to acception connection: {}", e);
                continue;
            }
        };

        info!("Accepted new connection from {}", addr);

        let sender_clone = sender.clone();
        let mut rx_sub = broadcast_channel.subscribe();

        // tokio::spawn(async move {
        //     loop {
        //         let data = rx_sub.recv().await.expect("Failed to received from lib");
        //         info!("Sending {} to {}", data, addr);
        //         if let Err(e) = socket.write_all(&data.into_bytes()).await {
        //             error!("Failed to write pong to socket {}: {}", addr, e);
        //             return;
        //         }
        //     }
        // });

        tokio::spawn(async move {
            let mut buf = vec![0; BUFFER_SIZE];
            let mut segmented_message = String::new();
            let sender_to_main = sender_clone.clone();
            loop {
                debug!("Waiting to read from {}...", addr);
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        info!("Connection closed by {}", addr);
                        return;
                    }
                    Ok(n) => {
                        let messages_str = match str::from_utf8(&buf[0..n]) {
                            Ok(v) => v,
                            Err(e) => {
                                error!("Unvalid utf-8 encoding on message: {}", e);
                                error!("   {:?}", buf);
                                continue;
                            }
                        };

                        let part = String::from(messages_str);
                        segmented_message.push_str(&part);

                        if !segmented_message.ends_with("\0") {
                            debug!(
                                "Segmented message currently of size: {}: {}",
                                segmented_message.len(),
                                segmented_message
                            );
                            continue;
                        }

                        let messages = segmented_message.split("\0").filter(|m| m.len() > 0);

                        let mc = messages.clone().count();

                        if mc > 1 {
                            debug!("Multipart message handled: {}", mc);
                        }

                        let mut i = 1;
                        for m in messages {
                            let sz = m.len();
                            debug!("{} Prt {}/{} ({} chars): {}", addr, i, mc, sz, m);
                            handle_message(m, &mut socket, addr).await;
                            let _ = sender_to_main.send(String::from(m)).map_err(|e| {
                                error!("Failed to send adapter data to main: {}", e);
                            });
                            i += 1;
                        }
                        segmented_message.clear();
                    }
                    Err(e) => {
                        error!("Failed to read from socket {}: {}", addr, e);
                        return;
                    }
                }
            }
        });
    }
}
