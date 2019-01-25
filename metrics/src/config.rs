// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

use serde_json;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub socket_path: String, // The path to the socket we listen on.
    pub mqtt_host: String,   // The url of the mqtt server.
    pub buffer_size: usize,  // The number of events we keep.
    pub relay_port: u16,     // The socket port we relay packets to.
    pub verbose: bool,       // True to display debug logs.
}

impl Config {
    pub fn load(path: &PathBuf) -> Self {
        let mut file = File::open(path).expect("Can't open config file");
        let mut source = String::new();
        file.read_to_string(&mut source)
            .expect("Unable to read config file");
        serde_json::from_str(&source).expect("Invalid config file")
    }
}

#[test]
fn load_config() {
    let config = Config::load(&PathBuf::from("./config.json.sample"));
    assert_eq!(config.mqtt_host, "localhost:12345");
}
