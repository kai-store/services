// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

/// Messages exchanged by the daemon with clients.

use chrono::{Timelike, Utc};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize, Default)]
pub struct SuccessFrame {
    pub success: bool,
    pub seq_number: u64,
}

#[derive(Deserialize, Serialize, Default)]
pub struct ErrorFrame {
    pub success: bool,
    pub seq_number: u64,
    pub error: String,
}

// These frames are relayed from the DC apps to the
// modem data collectors to specify event filtering.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct FilterFrame {
    #[serde(rename = "NC")]
    pub nc: u64,
    #[serde(rename = "ND")]
    pub nd: u64,
    #[serde(rename = "NE")]
    pub ne: u64,
}

pub type SharedFilterFrame = Arc<Mutex<Cell<FilterFrame>>>;

impl Default for FilterFrame {
    // Default values are set to {"ND": 0x7FFFFFFF, "NE": 0x7FFFFFFF, "NC": 0x7FFFFFFF}
    fn default() -> Self {
        FilterFrame {
            nc: 0x7FFFFFFF,
            nd: 0x7FFFFFFF,
            ne: 0x7FFFFFFF,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterAck {
    pub kind: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>, // Optional error description.
}

impl Default for FilterAck {
    fn default() -> Self {
        FilterAck {
            kind: "FilterAck".into(),
            success: true,
            reason: None,
        }
    }
}

pub fn default_shared_filterframe() -> SharedFilterFrame {
    Arc::new(Mutex::new(Cell::new(FilterFrame::default())))
}

type CounterValue = Option<u32>;
type OString = Option<String>;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct ClientPayload {
    // Event name
    #[serde(rename = "Name")]
    pub name: String,

    // Common
    pub DT: OString, // Time of collecting the data parameters DD:MM:YY hh:mm
    #[serde(skip_serializing_if = "Option::is_none")]
    DI1: OString, // IMEI of the device
    #[serde(skip_serializing_if = "Option::is_none")]
    DI2: OString, // IMSI of the SIM used
    #[serde(skip_serializing_if = "Option::is_none")]
    DI3: OString, // MSISDN of the device
    #[serde(skip_serializing_if = "Option::is_none")]
    DI4: OString, // Phone model reporting the data set
    #[serde(skip_serializing_if = "Option::is_none")]
    DI5: OString, // SW version used in the device
    #[serde(skip_serializing_if = "Option::is_none")]
    LI1: Option<u32>, // MCC/MNC
    #[serde(skip_serializing_if = "Option::is_none")]
    LI2: Option<u32>, // Tracking Area Code as seen by the device
    #[serde(skip_serializing_if = "Option::is_none")]
    LI3: Option<u32>, // Global Cell identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    LI4: Option<u16>, // Physical cell identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    LI5: Option<f64>, // Latitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    LI6: Option<f64>, // Longitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    LI7: Option<bool>, // Indicates GPS collected or not,
    #[serde(skip_serializing_if = "Option::is_none")]
    LI8: Option<f64>, // Indicates the accuracy of GPS coordinates,
    #[serde(skip_serializing_if = "Option::is_none")]
    SI1: Option<u8>, // Battery Level of the device
    #[serde(skip_serializing_if = "Option::is_none")]
    SI2: Option<u8>, // CPU Usage in percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    SI3: Option<u8>, // Memory usage in percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    TI1: Option<i32>, // Device Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    TI2: Option<i32>, // Battery Temperature

    // Network
    #[serde(skip_serializing_if = "Option::is_none")]
    RI1: Option<u8>, // RSRP as reported by the device. Represents -140 dBm to -43dBm
    #[serde(skip_serializing_if = "Option::is_none")]
    RI2: Option<u8>, // RSRQ as reported by the device. Represents -3dB to  -20dB
    #[serde(skip_serializing_if = "Option::is_none")]
    RI3: Option<i8>, // Signal to Interference plus Noise Ratio in dB
    #[serde(skip_serializing_if = "Option::is_none")]
    RI4: Option<u8>, // Channel Quality Indicator as derived by the device
    #[serde(skip_serializing_if = "Option::is_none")]
    RI5: Option<u8>, // Rank indicator when MIMO used
    #[serde(skip_serializing_if = "Option::is_none")]
    RI6: Option<u8>, // Current band used by the device
    #[serde(skip_serializing_if = "Option::is_none")]
    RI7: Option<u16>, // Frequency used by the device
    #[serde(skip_serializing_if = "Option::is_none")]
    RI8: Option<bool>, // Indicates if the device is out of service or in-service
    #[serde(skip_serializing_if = "Option::is_none")]
    RI9: OString, // Indicates the cause to initiate RRC Connection
    #[serde(skip_serializing_if = "Option::is_none")]
    RI10: Option<u16>, // Indicates the cause of the RRC connection release.
    #[serde(skip_serializing_if = "Option::is_none")]
    RI11: Option<i8>, // Maximum power used for the latest RACH transmission
    #[serde(skip_serializing_if = "Option::is_none")]
    RI12: Option<u8>, // Residual BLER at the physical layer
    #[serde(skip_serializing_if = "Option::is_none")]
    RI13: Option<u16>, // Current timing advance used by the device to communicate with the eNB.
    #[serde(skip_serializing_if = "Option::is_none")]
    RI14: Option<i8>, // Transmit power of the device at the time of reading
    #[serde(skip_serializing_if = "Option::is_none")]
    RI15: Option<Vec<(u32, u32, u32)>>, // Neigbor cell information stored
    #[serde(skip_serializing_if = "Option::is_none")]
    NI1: Option<bool>, // Indicates if the device is in a roaming area or not
    #[serde(skip_serializing_if = "Option::is_none")]
    NI2: Option<u8>, // Indicates the attach failure causes
    #[serde(skip_serializing_if = "Option::is_none")]
    NI3: Option<u8>, // Indicates the TAC update failure causes
    #[serde(skip_serializing_if = "Option::is_none")]
    NI4: OString, // EPS bearer details.
    // Received Signal Time difference values (RSTD) between the serving cell and neighbor cells (3)
    #[serde(skip_serializing_if = "Option::is_none")]
    OI1: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    VI1: OString, // The current status of SIP registration
    #[serde(skip_serializing_if = "Option::is_none")]
    VI2: Option<u8>, // The reason for terminating the SIP session
    #[serde(skip_serializing_if = "Option::is_none")]
    VI3: OString, // This will contain muting events
    #[serde(skip_serializing_if = "Option::is_none")]
    VI4: Option<u8>, // RTP Packet Loss percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    VI5: Option<u64>, // Number of packets lost due to jitter loss
    #[serde(skip_serializing_if = "Option::is_none")]
    HI1: Option<u64>, // The number of received data bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    HI2: Option<u64>, // The number of transmitted data bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    NE1: OString, // Call trigger
    #[serde(skip_serializing_if = "Option::is_none")]
    NE2: OString, // Call attempt failure
    #[serde(skip_serializing_if = "Option::is_none")]
    NE3: OString, // Call established
    #[serde(skip_serializing_if = "Option::is_none")]
    NE4: OString, // Call disconnect
    #[serde(skip_serializing_if = "Option::is_none")]
    NE5: OString, // Call drop
    #[serde(skip_serializing_if = "Option::is_none")]
    NE6: OString, // Call Muting
    #[serde(skip_serializing_if = "Option::is_none")]
    NE7: OString, // SMS Sent
    #[serde(skip_serializing_if = "Option::is_none")]
    NE8: OString, // SMS received
    #[serde(skip_serializing_if = "Option::is_none")]
    NE9: OString, // VoLTE registration event
    #[serde(skip_serializing_if = "Option::is_none")]
    NE10: OString, // VoLTE connection lost
    #[serde(skip_serializing_if = "Option::is_none")]
    NE11: OString, // Autonomous data collector event
    #[serde(skip_serializing_if = "Option::is_none")]
    NE12: OString, // Out of service
    #[serde(skip_serializing_if = "Option::is_none")]
    NE13: OString, // In service
    #[serde(skip_serializing_if = "Option::is_none")]
    NE14: OString, // ATTACH failure
    #[serde(skip_serializing_if = "Option::is_none")]
    NE15: OString, // TAC update
    #[serde(skip_serializing_if = "Option::is_none")]
    NE16: OString, // (RSRP<-110 dBm)
    #[serde(skip_serializing_if = "Option::is_none")]
    NE17: OString, // RRC Connection Release
    #[serde(skip_serializing_if = "Option::is_none")]
    NE18: OString, // RRC Connection failure
    #[serde(skip_serializing_if = "Option::is_none")]
    NE19: OString, // Radio Link Failure
    #[serde(skip_serializing_if = "Option::is_none")]
    NE20: OString, // Intra frequency handover
    #[serde(skip_serializing_if = "Option::is_none")]
    NE21: OString, // Inter frequency handover
    #[serde(skip_serializing_if = "Option::is_none")]
    NE22: OString, // Inter band handover
    #[serde(skip_serializing_if = "Option::is_none")]
    NE23: OString, // Cell reselection
    #[serde(skip_serializing_if = "Option::is_none")]
    NE24: OString, // RACH failure
    #[serde(skip_serializing_if = "Option::is_none")]
    NE25: OString, // Data pause or recoverable data stall
    #[serde(skip_serializing_if = "Option::is_none")]
    NE26: OString, // Non-recoverable data stall
    #[serde(skip_serializing_if = "Option::is_none")]
    NC1: CounterValue, // Number of outgoing calls
    #[serde(skip_serializing_if = "Option::is_none")]
    NC2: CounterValue, // Number of incoming calls
    #[serde(skip_serializing_if = "Option::is_none")]
    NC3: CounterValue, // Number of call attempt failures
    #[serde(skip_serializing_if = "Option::is_none")]
    NC4: CounterValue, // Number of call drops
    #[serde(skip_serializing_if = "Option::is_none")]
    NC5: CounterValue, // Number of data sessions
    #[serde(skip_serializing_if = "Option::is_none")]
    NC6: CounterValue, // Number of data session attempts failed
    #[serde(skip_serializing_if = "Option::is_none")]
    NC7: CounterValue, // Number of ATTACHs
    #[serde(skip_serializing_if = "Option::is_none")]
    NC8: CounterValue, // Number of ATTACH failures
    #[serde(skip_serializing_if = "Option::is_none")]
    NC9: CounterValue, // Number of DETACHs
}

error_chain! {
    errors {
        InvalidRI6(v: u8) {
            description("InvalidRI6")
            display("Invalid RI6 value: : needs to be 40, 5, or 3 but is {}.", v)
        }

        EmptyName {
            description("EmptyName")
            display("The payload name is mandatory and can't be empty.")
        }
    }
}

impl ClientPayload {
    pub fn validate(mut self) -> Result<Self> {
        // RI6 range is {40, 5, 3}
        // We also allow 0 because the modem may not have the actual band value
        // during registration.
        if let Some(ri6) = self.RI6 {
            if ri6 != 0 && ri6 != 3 && ri6 != 5 && ri6 != 40 {
                bail!(ErrorKind::InvalidRI6(ri6));
            }
        }

        // Name must not be empty.
        if self.name.is_empty() {
            bail!(ErrorKind::EmptyName);
        }

        // Make sure that DT is set.
        if self.DT.is_none() {
            // We don't want sub-second precision
            self.DT = Some(format!("{:?}", Utc::now().with_nanosecond(0).unwrap()));
        }
        Ok(self)
    }

    #[cfg(test)]
    pub fn bad_ri6() -> Self {
        let mut payload = ClientPayload::default();
        payload.RI6 = Some(1);
        payload.name = "NE9".to_owned();

        payload
    }

    #[cfg(test)]
    pub fn test_ri12() -> Self {
        use serde_json;

        let input = r#"{ "Name":"NE10",
            "RI1":52,
            "RI2":13,
            "RI3":18,
            "RI6":3,
            "RI7":0,
            "RI8":true,
            "RI9":"LTE_RRC_EST_CAUSE_MO_DATA",
            "RI11": 16,
            "RI13": 0,
            "VI2": 31,
            "RI4": 12,
            "RI5": 0,
            "NI1": true,
            "NI2": 3,
            "VI1": "NOT REGISTER"}"#;

        serde_json::from_str(input).unwrap()
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct ClientMessage {
    pub timestamp: u64,
    pub seq_number: u64,
    pub payload: ClientPayload,
}

#[test]
fn sample_message() {
    use serde_json;

    let input = r#"{ "seq_number": 1, "timestamp": 1502889770,
    "payload": { "Name":"NE1",
    "RI1":45,
    "RI2":128,
    "RI3":116,
    "RI6":0,
    "RI7":1374,
    "RI8":true,
    "RI13": 0,
    "NI1":true,
    "LI2": 31,
    "VI2": 240,
    "VI1": "REGISTERED"}}"#;

    let message: ClientMessage = serde_json::from_str(input).unwrap();
    assert_eq!(message.seq_number, 1);
    assert_eq!(message.timestamp, 1502889770);
    assert_eq!(message.payload.name, "NE1");
}

#[test]
fn ri11_and_ri13() {
    use serde_json;

    let input = r#"{ "seq_number": 1, "timestamp": 1502889770,
    "payload": { "Name":"NE10",
    "RI1":52,
    "RI2":13,
    "RI3":18,
    "RI6":3,
    "RI7":0,
    "RI8":true,
    "RI9":"LTE_RRC_EST_CAUSE_MO_DATA",
    "RI11": 16,
    "RI13": 0,
    "VI2": 31,
    "RI4": 12,
    "RI5": 0,
    "NI1": true,
    "NI2": 3,
    "VI1": "NOT REGISTER"}}"#;

    let message: ClientMessage = serde_json::from_str(input).unwrap();
    assert_eq!(message.seq_number, 1);
    assert_eq!(message.timestamp, 1502889770);
    assert_eq!(message.payload.name, "NE10");
}

#[test]
fn ri12() {
    use serde_json;

    let input = r#"{ "seq_number": 1, "timestamp": 1522393388, "payload": {
        "Name":"NE17",
        "RI1":37,
        "RI2":8,
        "RI3":5,
        "RI4": 11,
        "RI5": 0,
        "RI6":40,
        "RI7":38775,
        "RI8":true,
        "RI9":"mo_Data",
        "RI11": 16,
        "RI12": 0,
        "VI2": 16,
        "NI1":true,
        "NI2": 0,
        "VI1": "REGISTERED"}}"#;

    let message: ClientMessage = serde_json::from_str(input).unwrap();
    assert_eq!(message.seq_number, 1);
    assert_eq!(message.timestamp, 1522393388);
    assert_eq!(message.payload.name, "NE17");
}

#[test]
fn ri11_ri14_negative() {
    use serde_json;

    let input = r#"{ "seq_number": 1, "timestamp": 1522393388, "payload": {
        "Name":"NE17",
        "RI1":37,
        "RI2":8,
        "RI3":5,
        "RI14": -41,
        "RI5": 0,
        "RI6":40,
        "RI7":38775,
        "RI8":true,
        "RI9":"mo_Data",
        "RI11": -16,
        "RI12": 0,
        "VI2": 16,
        "NI1":true,
        "NI2": 0,
        "VI1": "REGISTERED"}}"#;

    let message: ClientMessage = serde_json::from_str(input).unwrap();
    assert_eq!(message.seq_number, 1);
    assert_eq!(message.timestamp, 1522393388);
    assert_eq!(message.payload.RI14.unwrap(), -41);
    assert_eq!(message.payload.RI11.unwrap(), -16);
}

#[test]
fn full_payload() {
    use serde_json;

    // let mut payload = ClientPayload::default();
    // payload.RI15 = Some(vec![(1, 2, 3), (4, 5, 6)]);
    // let s = serde_json::to_string(&payload).unwrap();
    // assert_eq!(s, "heelo");

    let input = r#"{ "seq_number": 1, "timestamp": 1522393388, "payload": {
        "Name":"NE17",

        "DI1": "abcdefg",
        "DI2": "0127654",
        "DI3": "34fe987",
        "DI4": "kaios_phone_1",
        "DI5": "2.5_r1",
        "LI1": 234567,
        "LI2": 567,
        "LI3": 56789,
        "LI4": 1212,
        "LI5": 122.04,
        "LI6": 37.3,
        "LI7": true,
        "LI8": 1.0,
        "SI1": 57,
        "SI2": 31,
        "SI3": 67,
        "TI1": 40,
        "TI2": 43,

        "RI1":37,
        "RI2":8,
        "RI3":5,
        "RI4": 11,
        "RI5": 0,
        "RI6":40,
        "RI7":38775,
        "RI8":true,
        "RI9":"mo_Data",
        "RI10": 3,
        "RI11": 16,
        "RI12": 0,
        "RI13": 45,
        "RI14": 3,
        "RI15": [[1,2,3],[4,5,6]],
        "NI1": false,
        "NI2": 4,
        "NI3": 3,
        "NI4": "eps bearer",
        "OI1": [0.5, 0.2, 0.333],
        "VI1": "REGISTERED",
        "VI2": 16,
        "VI3": "[[10,20,30],[40,50,60]]",
        "VI4": 5,
        "HI1": 123456,
        "HI2": 45678,
        "NE1": "ne1 value",
        "NE2": "ne2 value",
        "NE3": "ne3 value",
        "NE4": "ne4 value",
        "NE5": "ne5 value",
        "NE6": "ne6 value",
        "NE7": "ne7 value",
        "NE8": "ne8 value",
        "NE9": "ne9 value",
        "NE10": "ne10 value",
        "NE11": "ne11 value",
        "NE12": "ne12 value",
        "NE13": "ne13 value",
        "NE14": "ne14 value",
        "NE15": "ne15 value",
        "NE16": "ne16 value",
        "NE17": "ne17 value",
        "NE18": "ne18 value",
        "NE19": "ne19 value",
        "NE20": "ne20 value",
        "NE21": "ne21 value",
        "NE22": "ne22 value",
        "NE23": "ne23 value",
        "NE24": "ne24 value",
        "NE25": "ne25 value",
        "NE26": "ne26 value",

        "NC1": 1,
        "NC2": 2,
        "NC3": 3,
        "NC4": 5,
        "NC5": 8,
        "NC6": 13,
        "NC7": 21,
        "NC8": 34,
        "NC9": 55
        }}"#;

    let message: ClientMessage = serde_json::from_str(input).unwrap();
    assert_eq!(message.seq_number, 1);
    assert_eq!(message.timestamp, 1522393388);
    assert_eq!(message.payload.name, "NE17");
}

#[test]
fn decode_filter() {
    use serde_json;

    // 0x07008081 = 117473409
    // 0x040003CC = 67109836
    // 0x00000003 = 3
    let input = r#"{"ND": 117473409, "NE": 67109836, "NC": 3}"#;

    let message: FilterFrame = serde_json::from_str(input).unwrap();
    assert_eq!(message.nc, 3);
}
