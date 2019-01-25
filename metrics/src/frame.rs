// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::io::{Read, Write};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FrameType {
    Invalid,
    Json,
}

impl From<u8> for FrameType {
    fn from(source: u8) -> Self {
        match source {
            1 => FrameType::Json,
            _ => FrameType::Invalid,
        }
    }
}

impl Into<u8> for FrameType {
    fn into(self) -> u8 {
        match self {
            FrameType::Invalid => 0,
            FrameType::Json => 1,
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub typ: FrameType,
    data: Vec<u8>,
}

error_chain!{
    errors {
        InvalidFrameType(t: u8) {
            description("Invalid frame type")
            display("Invalid frame type: '{}'", t)
        }

        Json(s: String)
    }

    foreign_links {
        Io(::std::io::Error);
    }
}

impl Frame {
    /// Tries to read a Frame from an io:Read implementation.
    pub fn read_from<T: Read>(source: &mut T) -> Result<Frame> {
        let mut typ: [u8; 1] = [0; 1];
        source.read(&mut typ)?;

        if FrameType::from(typ[0]) == FrameType::Invalid {
            bail!(ErrorKind::InvalidFrameType(typ[0]));
        }

        // Read the frame length as a unsigned 32 bits network order integer.
        let length = source.read_u32::<BigEndian>()?;

        let mut data = Vec::with_capacity(length as usize);
        unsafe {
            data.set_len(length as usize);
        }

        source.read(&mut data)?;
        Ok(Frame {
            typ: FrameType::from(typ[0]),
            data: data,
        })
    }

    /// Tries to write a Frame to an io:Write implementation.
    pub fn write_to<T: Write>(self, dest: &mut T) -> Result<()> {
        let typ: [u8; 1] = [self.typ.into()];
        dest.write_all(&typ)?;

        dest.write_u32::<BigEndian>(self.data.len() as u32)?;

        dest.write_all(&self.data)?;

        dest.flush()?;
        Ok(())
    }

    /// Build a Json frame from a Json value.
    pub fn from_json(json: &Value) -> Self {
        let s = json.to_string();

        Frame {
            typ: FrameType::Json,
            data: s.as_bytes().to_vec(),
        }
    }

    /// Get the Json value from a Json frame.
    pub fn json(&self) -> Result<Value> {
        if self.typ != FrameType::Json {
            bail!(ErrorKind::InvalidFrameType(self.typ.clone().into()));
        }

        match serde_json::from_slice(&self.data) {
            Ok(val) => Ok(val),
            Err(e) => {
                let json =
                    String::from_utf8(self.data.clone()).unwrap_or("Invalid utf8".to_owned());
                debug!("json() failed: payload is {}", json);
                Err(ErrorKind::Json(format!("{:?}", e)).into())
            }
        }
    }

    pub fn deserialize<'a, T>(&'a self) -> Result<T>
    where
        T: Deserialize<'a>,
    {
        if self.typ != FrameType::Json {
            bail!(ErrorKind::InvalidFrameType(self.typ.clone().into()));
        }

        match serde_json::from_slice(&self.data) {
            Ok(val) => Ok(val),
            Err(e) => {
                let json =
                    String::from_utf8(self.data.clone()).unwrap_or("Invalid utf8".to_owned());
                trace!("deserialize() failed: payload is {} : {}", json, e);
                Err(ErrorKind::Json(format!("{:?}", e)).into())
            }
        }
    }

    pub fn from_obj<T>(obj: &T) -> Self
    where
        T: Serialize,
    {
        Frame {
            typ: FrameType::Json,
            data: serde_json::to_vec(obj).unwrap(),
        }
    }
}

#[test]
fn test_from_json() {
    let value = json!({"result":true});

    let frame = Frame::from_json(&value);
    assert_eq!(frame.typ, FrameType::Json);
    assert_eq!(frame.data.len(), 15);
}

#[test]
fn test_as_valid_json() {
    let frame = Frame {
        typ: FrameType::Json,
        data: b"{\"result\":true}".to_vec(),
    };

    let value = frame.json();
    assert_eq!(value.is_ok(), true);
}

#[test]
fn test_as_valid_json_invalid_frame_type() {
    let frame = Frame {
        typ: FrameType::Invalid,
        data: b"{\"result\":true}".to_vec(),
    };

    let value = frame.json();
    assert_eq!(value.is_err(), true);
    assert_eq!(format!("{:?}", value.err().unwrap()),
    "Error(InvalidFrameType(0), State { next_error: None })");
}

#[test]
fn test_as_invalid_json() {
    let frame = Frame {
        typ: FrameType::Json,
        data: vec![0, 1, 2, 3, 4, 5],
    };

    let value = frame.json();
    assert_eq!(value.is_err(), true);
    assert_eq!(format!("{:?}", value.err().unwrap()),
        "Error(Json(\"Error(\\\"expected value\\\", line: 1, column: 1)\"), \
    State { next_error: None })");
}

#[test]
fn test_deserialize_valid() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct InitFrame {
        source: String,
    }

    let frame = Frame {
        typ: FrameType::Json,
        data: b"{\"source\":\"test_valid\"}".to_vec(),
    };

    let init_frame: InitFrame = frame.deserialize().unwrap();
    assert_eq!(init_frame.source, "test_valid");
}

#[test]
fn test_deserialize_invalid_frame_type() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct InitFrame {
        source: String,
    }

    let frame = Frame {
        typ: FrameType::Invalid,
        data: b"{\"source\":\"test_valid\"}".to_vec(),
    };

    let init_frame: Result<InitFrame> = frame.deserialize();
    assert_eq!(format!("{:?}", init_frame.err().unwrap()),
    "Error(InvalidFrameType(0), State { next_error: None })");
}

#[test]
fn test_deserialize_invalid_data() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct InitFrame {
        source: String,
    }

    let frame = Frame {
        typ: FrameType::Json,
        data: b"{\"source2\":\"test_valid\"}".to_vec(),
    };

    let init_frame: Result<InitFrame> = frame.deserialize();
    assert_eq!(init_frame.is_err(), true);
}
