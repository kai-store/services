// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

use config::Config;
use frame::{Frame, Result as FrameResult};
use frame_messages::{ClientMessage, ErrorFrame, FilterAck, SharedFilterFrame, SuccessFrame};
use internal_messages::InternalMessage;
use libc;
use message_broker::SharedMessageBroker;
use std::collections::HashSet;
use std::ffi::CString;
use std::fs;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
// use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

type SharedSources = Arc<Mutex<HashSet<String>>>;

fn on_new_socket(
    source_stream: UnixStream,
    sources: SharedSources,
    broker: SharedMessageBroker<InternalMessage>,
    filter: SharedFilterFrame,
) {
    let mut source = String::new();

    let shared_stream = Arc::new(Mutex::new(source_stream));

    macro_rules! stry {
        ($e:expr) => {
            match $e {
                Err(e) => {
                    {
                        let mut guard = sources.lock().unwrap();
                        guard.remove(&source);
                    }
                    debug!("Closing connection: {:?} {}", e, source);
                    let stream = shared_stream.lock().unwrap();
                    stream
                        .shutdown(Shutdown::Both)
                        .expect("shutdown function failed");
                    return;
                }
                Ok(val) => val,
            }
        };
    }

    macro_rules! read_frame {
        ($t:ty) => {{
            let mut stream = shared_stream.lock().unwrap();
            match Frame::read_from(&mut *stream) {
                Err(err) => {
                    {
                        let mut guard = sources.lock().unwrap();
                        guard.remove(&source);
                    }
                    debug!(
                        "Invalid frame ({:?}), closing connection for {}",
                        err, source
                    );
                    stream
                        .shutdown(Shutdown::Both)
                        .expect("shutdown function failed");
                    return;
                }
                Ok(val) => {
                    let obj: FrameResult<$t> = val.deserialize();
                    stry!(obj)
                }
            }
        }};
    }

    macro_rules! try_read_frame {
        ($t:ty) => {{
            let mut stream = shared_stream.lock().unwrap();
            match Frame::read_from(&mut *stream) {
                Err(err) => {
                    {
                        let mut guard = sources.lock().unwrap();
                        guard.remove(&source);
                    }
                    debug!(
                        "Invalid frame ({:?}), closing connection for {}",
                        err, source
                    );
                    stream
                        .shutdown(Shutdown::Both)
                        .expect("shutdown function failed");
                    return;
                }
                Ok(val) => {
                    let obj: FrameResult<$t> = val.deserialize();
                    (val, obj)
                }
            }
        }};
    }

    debug!("New socket");

    // Reads the first Frame, which needs to be in the { "source": "ril_metrics" } format.
    #[derive(Deserialize, Debug)]
    struct InitFrame {
        source: String,
    }
    let init_frame = read_frame!(InitFrame);

    source = init_frame.source;

    // Verify is this source is not already active.
    {
        let mut guard = sources.lock().unwrap();
        if guard.contains(&source) {
            debug!("Source already connected, closing connection");
            let stream = shared_stream.lock().unwrap();
            stream
                .shutdown(Shutdown::Both)
                .expect("shutdown function failed");
            return;
        } else {
            guard.insert(source.clone());
        }
    }

    info!("Accepting connection from {}", source);

    {
        let ack = Frame::from_json(&json!({ "ready": true }));
        let mut stream = shared_stream.lock().unwrap();
        stry!(ack.write_to(&mut *stream));
    }

    // Send the current filter.
    // {
    //     let filter = &(*filter.lock().unwrap()).get();
    //     let frame = Frame::from_obj(filter);
    //     let mut stream = shared_stream.lock().unwrap();
    //     stry!(frame.write_to(&mut *stream));
    //     debug!("Sent initial filter: {:?}", filter);
    // }

    // Register as a broker receive notices of changes in the filtering and
    // push it to the client.
    // let (tx, rx) = channel::<InternalMessage>();
    // {
    //     let mut guard = broker.lock().unwrap();
    //     guard
    //         .add_actor(
    //             &format!("client-{}-{}", source, sources.lock().unwrap().len()),
    //             tx.clone(),
    //         )
    //         .unwrap();
    // }

    // let shared_stream2 = shared_stream.clone();
    // let sources2 = sources.clone();
    // let source2 = source.clone();
    // thread::Builder::new()
    //     .name("listener filter".to_owned())
    //     .spawn(move || {
    //         macro_rules! stry2 {
    //             ($e:expr) => {
    //                 match $e {
    //                     Err(e) => {
    //                         {
    //                             let mut guard = sources2.lock().unwrap();
    //                             guard.remove(&source2);
    //                         }
    //                         debug!("Closing connection: {:?} {}", e, source2);
    //                         let stream = shared_stream2.lock().unwrap();
    //                         stream
    //                             .shutdown(Shutdown::Both)
    //                             .expect("shutdown function failed");
    //                         return;
    //                     }
    //                     Ok(val) => val,
    //                 }
    //             };
    //         }

    //         loop {
    //             let msg = rx.recv().unwrap();
    //             match msg {
    //                 InternalMessage::NewFilter(filter) => {
    //                     // send the message to the stream.
    //                     info!("About to send filter to RIL: {:?}", filter);
    //                     let frame = Frame::from_obj(&filter);
    //                     let mut stream = shared_stream2.lock().unwrap();
    //                     info!("Stream lock acquired");
    //                     stry2!(frame.write_to(&mut *stream));
    //                     info!("Updated filter: {:?}", filter);
    //                 }
    //                 InternalMessage::Shutdown => {
    //                     info!("Shutting down queue manager thread");
    //                     break;
    //                 }
    //                 _ => {
    //                     // Nothing to do with the other messages.
    //                 }
    //             }
    //         }
    //     })
    //     .expect("Failed to start filter receiving thread");

    // Once we have sent the readiness packet, loop while we get more
    // valid frames from the client, and dispatch them.
    let mut last_seq_number = 0u64;
    loop {
        // Check if this is a filter ack.
        let (val, filter) = try_read_frame!(FilterAck);
        if let Ok(filter_ack) = filter {
            // Relay to the socket and bail out of this loop iteration.
            debug!("FilterAck is {:?}", filter_ack);

            // Push the frame to the queue.
            stry!(
                broker
                    .lock()
                    .unwrap()
                    .send_message("queue", InternalMessage::FilterAck(filter_ack))
            );

            continue;
        } else {
            debug!("Not a filter ack");
        }

        let decoded: FrameResult<Vec<ClientMessage>> = val.deserialize();
        let messages = stry!(decoded);

        for message in messages {
            debug!(
                "Got frame from {}: seq={}, timestamp={}",
                source, message.seq_number, message.timestamp
            );
            if message.seq_number <= last_seq_number && message.seq_number != 1 {
                // Close the connection.
                debug!("Invalid seq_number, closing connection");
                let stream = shared_stream.lock().unwrap();
                stream
                    .shutdown(Shutdown::Both)
                    .expect("shutdown function failed");
                return;
            } else {
                last_seq_number = message.seq_number;
            }

            // Validate the payload.
            let payload = message.payload.validate();
            if let Err(err) = payload {
                debug!("Invalid payload: {}", err);

                // Send an error payload.
                let msg = ErrorFrame {
                    success: false,
                    seq_number: message.seq_number,
                    error: format!("{}", err.description()),
                };
                let frame = Frame::from_obj(&msg);

                let mut stream = shared_stream.lock().unwrap();
                stry!(frame.write_to(&mut *stream));
                continue;
            }

            // Push the frame to the queue.
            stry!(
                broker
                    .lock()
                    .unwrap()
                    .send_message("queue", InternalMessage::NewClientMessage(payload.unwrap()))
            );

            // Return a success message.
            let msg = SuccessFrame {
                success: true,
                seq_number: message.seq_number,
            };
            let frame = Frame::from_obj(&msg);

            {
                let mut stream = shared_stream.lock().unwrap();
                stry!(frame.write_to(&mut *stream));
            }
        }
    }
}

pub fn start_listener(
    config: &Config,
    broker: SharedMessageBroker<InternalMessage>,
    filter: SharedFilterFrame,
) {
    debug!(
        "Starting the metrics socket endpoint at {:?}",
        config.socket_path
    );
    let config = config.clone();

    let sources = Arc::new(Mutex::new(HashSet::<String>::new()));

    thread::Builder::new()
        .name("socket listener".to_owned())
        .spawn(move || {
            let spath = config.socket_path.clone();
            if Path::exists(Path::new(&config.socket_path)) {
                #[allow(unused_must_use)]
                {
                    fs::remove_file(spath.clone());
                }
            }

            let path = PathBuf::from(spath.clone());

            let socket = match UnixListener::bind(path) {
                Ok(sock) => sock,
                Err(e) => {
                    error!("Couldn't bind: {:?}", e);
                    return;
                }
            };

            // chmod the socket to 660
            let cpath = CString::new(spath.clone()).unwrap();
            if unsafe { libc::chmod(cpath.as_ptr(), 0o660) } != 0 {
                // Unfortunately the libc crate doesn't expose errno :(
                error!("Failed to chmod 0660 {}", spath);
            } else {
                info!("Successfully chmod 0660 {}", spath);
            }

            for stream in socket.incoming() {
                match stream {
                    Ok(stream) => {
                        let s = sources.clone();
                        let b = broker.clone();
                        let f = filter.clone();
                        thread::spawn(move || on_new_socket(stream, s, b, f));
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        })
        .expect("Failed to start socket listener thread");
}

#[test]
fn test_listener() {
    use chrono::{Timelike, Utc};
    use frame_messages::{default_shared_filterframe, ClientPayload, FilterFrame};
    use message_broker::MessageBroker;
    use std::io::Read;
    use std::net::TcpListener;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (tx, rx) = channel();

    let listener = TcpListener::bind("127.0.0.1:12345").expect("Failed to create listener socket");

    thread::Builder::new()
        .name("test listener".to_owned())
        .spawn(move || {
            // Wait a bit to let the listener be setup.
            thread::sleep(Duration::new(1, 0));

            let mut relay = listener.incoming().next().unwrap().unwrap();

            // Connect to the unix socket.
            let mut stream = UnixStream::connect("/tmp/metrics_daemon_1").unwrap();

            // Send the source frame and wait for ack.
            let init = Frame::from_json(&json!({ "source": "test_source" }));
            init.write_to(&mut stream).unwrap();
            #[derive(Deserialize)]
            struct AckFrame {
                ready: bool,
            }
            let ack: AckFrame = Frame::read_from(&mut stream)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(ack.ready, true);

            // let filter: FilterFrame = Frame::read_from(&mut stream)
            //     .unwrap()
            //     .deserialize()
            //     .unwrap();
            // assert_eq!(filter, FilterFrame::default());

            // Send the filter ack.
            let ack = Frame::from_json(&json!({ "success": true, "kind": "FilterAck" }));
            ack.write_to(&mut stream).unwrap();

            let now = format!("{:?}", Utc::now().with_nanosecond(0).unwrap());

            // Read the filter ack on the relay socket.
            let mut v = [0u8; 37];
            relay.read_exact(&mut v).unwrap();
            let json = String::from_utf8(v.to_vec()).unwrap();
            assert_eq!(
                r#"{"kind":"FilterAck","success":true}
 "#,
                json
            );
            // Send an empty frame and wait for success.
            let mut msg1 = ClientMessage::default();
            msg1.seq_number = 1;
            msg1.timestamp = 9999997;
            msg1.payload.name = "NE9".to_owned();
            let mut msg2 = ClientMessage::default();
            msg2.seq_number = 2;
            msg2.timestamp = 9999999;
            msg2.payload.name = "NE8".to_owned();
            let frame = Frame::from_obj(&vec![msg1, msg2]);
            frame.write_to(&mut stream).unwrap();
            let res: SuccessFrame = Frame::read_from(&mut stream)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(res.success, true);
            assert_eq!(res.seq_number, 1);
            let res: SuccessFrame = Frame::read_from(&mut stream)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(res.success, true);
            assert_eq!(res.seq_number, 2);

            // Missing name
            let mut msg3 = ClientMessage::default();
            msg3.seq_number = 3;
            msg3.timestamp = 9999999;
            let frame = Frame::from_obj(&vec![msg3]);
            frame.write_to(&mut stream).unwrap();
            let res: ErrorFrame = Frame::read_from(&mut stream)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(res.success, false);
            assert_eq!(res.seq_number, 3);
            assert_eq!(res.error, "EmptyName".to_owned());

            // Invalid RI6 value
            let mut msg4 = ClientMessage::default();
            msg4.seq_number = 4;
            msg4.timestamp = 9999999;
            msg4.payload = ClientPayload::bad_ri6();
            let frame = Frame::from_obj(&vec![msg4]);
            frame.write_to(&mut stream).unwrap();
            let res: ErrorFrame = Frame::read_from(&mut stream)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(res.success, false);
            assert_eq!(res.seq_number, 4);
            assert_eq!(res.error, "InvalidRI6".to_owned());

            // Read the JSON on the relay socket.
            // These are JSON strings with a '\n ' delimiter.
            let mut v = [0u8; 44];
            relay.read_exact(&mut v).unwrap();
            let json = String::from_utf8(v.to_vec()).unwrap();
            assert_eq!(
                format!(
                    r#"{{"Name":"NE9","DT":"{}"}}
 "#,
                    now
                ),
                json
            );

            relay.read_exact(&mut v).unwrap();
            let json = String::from_utf8(v.to_vec()).unwrap();
            assert_eq!(
                format!(
                    r#"{{"Name":"NE8","DT":"{}"}}
 "#,
                    now
                ),
                json
            );

            // Test RI12
            let mut msg5 = ClientMessage::default();
            msg5.seq_number = 5;
            msg5.timestamp = 9999999;
            msg5.payload = ClientPayload::test_ri12();
            let frame = Frame::from_obj(&vec![msg5]);
            frame.write_to(&mut stream).unwrap();
            let res: SuccessFrame = Frame::read_from(&mut stream)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(res.success, true);
            assert_eq!(res.seq_number, 5);

            // Signals that this thread is done.
            tx.send(()).unwrap();
        })
        .expect("Failed to create test thread");

    let config = Config {
        socket_path: "/tmp/metrics_daemon_1".to_owned(),
        mqtt_host: "localhost".to_owned(),
        buffer_size: 10,
        relay_port: 12345,
        verbose: false,
    };

    let broker: SharedMessageBroker<InternalMessage> = MessageBroker::new_shared();
    // Start with the default filter.
    let filter = default_shared_filterframe();

    ::queue::start_queue_manager(&config, broker.clone(), filter.clone());
    start_listener(&config, broker.clone(), filter);

    // Wait for the listener thread to be done.
    rx.recv().unwrap();

    broker
        .lock()
        .unwrap()
        .broadcast_message(InternalMessage::Shutdown);
}

#[test]
fn test_sources() {
    use frame_messages::default_shared_filterframe;
    use message_broker::MessageBroker;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (tx, rx) = channel();

    thread::Builder::new()
        .name("test listener".to_owned())
        .spawn(move || {
            // Wait a bit to let the listener be setup.
            thread::sleep(Duration::new(1, 0));

            // Connect to the unix socket.
            let mut stream1 = UnixStream::connect("/tmp/metrics_daemon_2").unwrap();

            // Send the source frame and wait for ack.
            let init = Frame::from_json(&json!({ "source": "test_source" }));
            init.write_to(&mut stream1).unwrap();
            #[derive(Deserialize)]
            struct AckFrame {
                ready: bool,
            }
            let ack: AckFrame = Frame::read_from(&mut stream1)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(ack.ready, true);

            // Connect again to the unix socket.
            let mut stream2 = UnixStream::connect("/tmp/metrics_daemon_2").unwrap();

            // Send the source frame and wait for ack.
            // This will be rejected because we already have such a source.
            let init = Frame::from_json(&json!({ "source": "test_source" }));
            init.write_to(&mut stream2).unwrap();
            let res = Frame::read_from(&mut stream2);
            assert_eq!(res.is_err(), true);

            // Connect to the unix socket.
            let mut stream3 = UnixStream::connect("/tmp/metrics_daemon_2").unwrap();

            // Send the source frame with a different source and wait for ack.
            let init = Frame::from_json(&json!({ "source": "test_source_2" }));
            init.write_to(&mut stream3).unwrap();
            let ack: AckFrame = Frame::read_from(&mut stream3)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(ack.ready, true);

            // Close the first stream, and then check that we can open it again with the
            // same source name.
            stream1
                .shutdown(Shutdown::Both)
                .expect("shutdown of stream1 failed");
            let mut stream4 = UnixStream::connect("/tmp/metrics_daemon_2").unwrap();

            // Send the source frame and wait for ack.
            let init = Frame::from_json(&json!({ "source": "test_source" }));
            init.write_to(&mut stream4).unwrap();
            let ack: AckFrame = Frame::read_from(&mut stream4)
                .unwrap()
                .deserialize()
                .unwrap();
            assert_eq!(ack.ready, true);

            // Signals that this thread is done.
            tx.send(()).unwrap();
        })
        .expect("Failed to create test thread");

    let config = Config {
        socket_path: "/tmp/metrics_daemon_2".to_owned(),
        mqtt_host: "localhost".to_owned(),
        buffer_size: 10,
        relay_port: 54321,
        verbose: false,
    };

    let broker: SharedMessageBroker<InternalMessage> = MessageBroker::new_shared();
    // Start with the default filter.
    let filter = default_shared_filterframe();

    ::queue::start_queue_manager(&config, broker.clone(), filter.clone());
    start_listener(&config, broker.clone(), filter);

    // Wait for the listener thread to be done.
    rx.recv().unwrap();

    broker
        .lock()
        .unwrap()
        .broadcast_message(InternalMessage::Shutdown);
}
