// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

/// Message queue manager.
use config::Config;
use frame_messages::{ClientPayload, FilterAck, SharedFilterFrame};
use internal_messages::InternalMessage;
use message_broker::SharedMessageBroker;
use serde::{Serialize, Serializer};
use socket_relay::{start_relay, SocketRelay};
use std::collections::VecDeque;
use std::sync::mpsc::channel;
use std::thread;

enum QueueItem {
    ClientPayload(ClientPayload),
    FilterAck(FilterAck),
}

impl Serialize for QueueItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            QueueItem::ClientPayload(ref val) => val.serialize(serializer),
            QueueItem::FilterAck(ref val) => val.serialize(serializer),
        }
    }
}

pub fn start_queue_manager(
    config: &Config,
    broker: SharedMessageBroker<InternalMessage>,
    filter: SharedFilterFrame,
) {
    let (tx, rx) = channel::<InternalMessage>();
    {
        let mut guard = broker.lock().unwrap();
        guard.add_actor("queue", tx.clone()).unwrap();
    }
    let max_queue_size = config.buffer_size;
    start_relay(&config, broker.clone(), filter.clone());

    thread::Builder::new()
        .name("queue manager".to_owned())
        .spawn(move || {
            let mut queue = VecDeque::<QueueItem>::new();
            let mut relay: Option<SocketRelay> = None;

            loop {
                let msg = rx.recv().unwrap();
                match msg {
                    InternalMessage::NewClientMessage(payload) => {
                        if let Some(ref mut relay) = relay {
                            debug!("Queue relay socket is ready, sending payload");
                            relay.send(&payload).expect("Failed to relay payload");
                        } else {
                            // Add to queue if the relay is not up.

                            if queue.len() == max_queue_size {
                                // The queue is full, evict the oldest element.
                                info!("Queue overflow, removing element");
                                queue.pop_front();
                            }
                            info!("Adding element to queue, size is now {}", queue.len());
                            queue.push_back(QueueItem::ClientPayload(payload));
                        }
                    }
                    InternalMessage::RelayReady(mut socket) => {
                        // Drain the queue.
                        debug!(
                            "Queue relay socket is ready, about to drain {} items",
                            queue.len()
                        );
                        for item in queue.drain(..) {
                            socket.send(&item).expect("Failed to relay drained payload");
                        }

                        relay = Some(socket);
                    }
                    InternalMessage::Shutdown => {
                        info!("Shutting down queue manager thread");
                        break;
                    }
                    InternalMessage::FilterAck(filter_ack) => {
                        // Send the ack packet to the socket.
                        // Since we send an initial filter packet, we can't be sure the relay is ready
                        // when we get the ack, so we buffer here to if needed.
                        if let Some(ref mut relay) = relay {
                            debug!("Queue relay socket is ready, sending filter ack");
                            relay.send(&filter_ack).expect("Failed to relay filter ack");
                        } else {
                            // Add to queue if the relay is not up.

                            if queue.len() == max_queue_size {
                                // The queue is full, evict the oldest element.
                                info!("Queue overflow, removing element");
                                queue.pop_front();
                            }
                            info!("Adding element to queue, size is now {}", queue.len());
                            queue.push_back(QueueItem::FilterAck(filter_ack));
                        }
                    }
                    InternalMessage::NewFilter(_) => {
                        // Nothing to do here.
                    }
                }
            }
        }).expect("Failed to create queue manager thread");
}
