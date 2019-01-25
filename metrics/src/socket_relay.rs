// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

/// Socket relay: this allows sending frames on a TCP Socket.
use config::Config;
use frame_messages::{FilterFrame, SharedFilterFrame};
use internal_messages::InternalMessage;
use message_broker::SharedMessageBroker;
use serde::Serialize;
use serde_json;
use std::fmt;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::result::Result as StdResult;
use std::{thread, time};

error_chain!{
    errors {
        NotReady {
            description("The socket is not ready yet")
            display("The socket is not ready yet")
        }
    }

    foreign_links {
        Io(::std::io::Error);
    }
}

pub struct SocketRelay {
    stream: TcpStream,
    filter: SharedFilterFrame,
    broker: SharedMessageBroker<InternalMessage>,
}

impl fmt::Debug for SocketRelay {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        write!(f, "SocketRelay stream:{:?}", self.stream)
    }
}

impl Clone for SocketRelay {
    fn clone(&self) -> Self {
        SocketRelay {
            stream: self
                .stream
                .try_clone()
                .expect("Failed to clone SocketRelay stream"),
            filter: self.filter.clone(),
            broker: self.broker.clone(),
        }
    }
}

impl SocketRelay {
    pub fn send<T: Serialize>(&mut self, payload: &T) -> Result<()> {
        // The client expects a JSON string with a \n ending, not a frame.
        let mut v = serde_json::to_vec(payload).unwrap();
        v.push(b'\n');
        v.push(b' ');
        self.stream.write_all(&v)?;
        self.stream.flush().map_err(|e| e.into())
    }

    // Wait for Json string carrying the new filter.
    pub fn listen_for_filter(&mut self) {
        // We want blocking reads.
        self.stream
            .set_read_timeout(None)
            .expect("Failed to set read timeout");
        loop {
            // Read one line of text.
            let buf_socket = BufReader::new(&self.stream);
            let data: Vec<_> = buf_socket
                .bytes()
                .take_while(|b| match *b {
                    Err(_) | Ok(b'\n') | Ok(b'\r') => false,
                    _ => true,
                })
                .map(|e| e.unwrap())
                .collect();
            let s = String::from_utf8_lossy(&data);
            debug!("Reading filter from DC App: |{}|", s);
            let new_filter: FilterFrame = match serde_json::from_str(&s) {
                Err(err) => {
                    error!("Invalid filter: {}", err);
                    continue;
                }
                Ok(v) => v,
            };
            debug!("Received filter from DC App: {:?}", new_filter);
            // Mutate the shared filter to have the new default picked up by incoming clients.
            self.filter.lock().unwrap().set(new_filter);
            // Dispatch a NewFilter event for already running clients.
            self.broker
                .lock()
                .unwrap()
                .broadcast_message(InternalMessage::NewFilter(new_filter.clone()));
        }
    }
}

pub fn start_relay(
    config: &Config,
    broker: SharedMessageBroker<InternalMessage>,
    filter: SharedFilterFrame,
) {
    // Tries to connect to a socket, and sends it back when it's ready.
    let port = config.relay_port;
    let filter = filter.clone();
    thread::Builder::new()
        .name("queue manager".to_owned())
        .spawn(move || loop {
            let mut delay = 1u64;
            debug!("Trying to connect to the socket on port {}", port);
            match TcpStream::connect(format!("127.0.0.1:{}", port)) {
                Err(_) => {
                    delay = delay * 2;
                    if delay > 10 {
                        delay = 10;
                    }
                    thread::sleep(time::Duration::new(delay, 0));
                }
                Ok(stream) => {
                    debug!("Connection established");
                    let filter = filter.clone();
                    let mut relay = SocketRelay {
                        stream,
                        filter,
                        broker: broker.clone(),
                    };
                    broker
                        .lock()
                        .unwrap()
                        .send_message("queue", InternalMessage::RelayReady(relay.clone()))
                        .expect("Failed to send socket relay");

                    // relay.listen_for_filter();
                    break;
                }
            }
            debug!("Shuting down relay startup thread.");
        })
        .expect("Failed to create socket relay thread");
}
