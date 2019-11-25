use std::error::Error as StdError;
use std::fmt::{self, Write};
use std::io::{self, Read};
use std::str::{FromStr, Utf8Error};

use bytes::{BufMut, BytesMut, IntoBuf};
use tokio::codec::{Decoder, Encoder};

use super::command::{Command, ParseError};
use super::message::Message;

pub struct IrcCodec;

#[derive(Debug)]
pub enum CodecError {
    Io(io::Error),
    Utf8(Utf8Error),
    Parse(ParseError),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl From<io::Error> for CodecError {
    fn from(err: io::Error) -> Self {
        CodecError::Io(err)
    }
}

impl From<Utf8Error> for CodecError {
    fn from(err: Utf8Error) -> Self {
        CodecError::Utf8(err)
    }
}

impl From<ParseError> for CodecError {
    fn from(err: ParseError) -> Self {
        CodecError::Parse(err)
    }
}

impl StdError for CodecError {}

impl Encoder for IrcCodec {
    type Item = Message;
    type Error = CodecError;

    fn encode(&mut self, item: Self::Item, bytes: &mut BytesMut) -> Result<(), Self::Error> {
        let message = format!("{}", item);
        bytes.reserve(message.len());
        bytes.put(message);
        println!("Bytes: {:?}", bytes);
        Ok(())
    }
}

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, bytes: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            let line_end = bytes
                .as_ref()
                .windows(2)
                .enumerate()
                .find_map(|(pos, slice)| {
                    if slice[0] == b'\r' && slice[1] == b'\n' {
                        Some(pos)
                    } else {
                        None
                    }
                });

            if let Some(offset) = line_end {
                let string = std::str::from_utf8(&bytes.as_ref()[..offset])
                    .map_err(CodecError::from)?
                    .to_owned();
                // skip empty strings
                if string.len() == 0 {
                    continue;
                }
                let message = Message::from_str(&string).map_err(CodecError::from)?;
                bytes.advance(offset + 2);
                return Ok(Some(message));
            } else {
                return Ok(None);
            }
        }
    }
}
