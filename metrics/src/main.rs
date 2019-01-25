// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

#[cfg(target_os = "android")]
extern crate android_logger;
extern crate byteorder;
extern crate chrono;
#[cfg(not(target_os = "android"))]
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate libc;
#[macro_use]
extern crate log;
extern crate mio;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod config;
pub mod frame;
pub mod frame_messages;
pub mod internal_messages;
pub mod listener;
pub mod message_broker;
pub mod queue;
pub mod socket_relay;

use config::Config;
use frame_messages::default_shared_filterframe;
use internal_messages::InternalMessage;
use libc::{getpid, sighandler_t, SIGINT};
use message_broker::{MessageBroker, SharedMessageBroker};
use mio::{Events, Poll};
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

// Handle SIGINT (Ctrl-C) for manual shutdown.
// Signal handlers must not do anything substantial. To trigger shutdown, we atomically
// flip this flag; the event loop checks the flag and exits accordingly.
static SHUTDOWN_FLAG: AtomicBool = ATOMIC_BOOL_INIT;
unsafe fn handle_sigint(_: i32) {
    SHUTDOWN_FLAG.store(true, Ordering::Release);
}

static VERSION : &'static str = include_str!("version.in");

#[cfg(target_os = "android")]
fn init_logger(verbose: bool) {
    use android_logger::Filter;
    use log::Level;

    let level = if verbose {
        Filter::default()
            .with_min_level(Level::Debug)
    } else {
        Filter::default()
            .with_min_level(Level::Info)
    };
    android_logger::init_once(level, Some("MetricsDaemon"));
}

#[cfg(not(target_os = "android"))]
fn init_logger(_verbose: bool) {
    env_logger::init();
}


fn main() {
    unsafe {
        libc::signal(SIGINT, handle_sigint as sighandler_t);
    }

    let cpath = match env::args().skip(1).next() {
        Some(val) => val,
        None => "./config.json".to_owned(),
    };

    
    let config = Config::load(&PathBuf::from(cpath.clone()));
    init_logger(config.verbose);
    
    info!("Starting metrics daemon {}, pid is {}", VERSION, unsafe { getpid() });
    info!("Loaded configuration file from {}, verbose mode: {}", cpath, config.verbose);


    let broker: SharedMessageBroker<InternalMessage> = MessageBroker::new_shared();
    // Start with the default filter.
    let filter = default_shared_filterframe();

    queue::start_queue_manager(&config, broker.clone(), filter.clone());
    listener::start_listener(&config, broker.clone(), filter);

    // Simple event loop, just waiting for SIGINT to exit.
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    loop {
        let _ = poll.poll(&mut events, None);
        if SHUTDOWN_FLAG.load(Ordering::Acquire) {
            break;
        }
    }

    info!("Starting shutdown of metrics daemon");
    broker
        .lock()
        .unwrap()
        .broadcast_message(InternalMessage::Shutdown);
    ::std::thread::sleep(::std::time::Duration::new(1, 0));
    info!("Shutdown complete.");
}
