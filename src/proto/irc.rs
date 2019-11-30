//! Implementation of IRC codec for Tokio.
use anyhow::{Error, Result};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LinesCodec};

use crate::proto::message::Message;

/// An IRC codec built around an inner codec.
pub struct IrcCodec {
    inner: LinesCodec,
}

impl IrcCodec {
    /// Creates a new instance of IrcCodec wrapping a LineCodec with the specific encoding.
    pub fn new() -> IrcCodec {
        IrcCodec {
            inner: LinesCodec::new(),
        }
    }

    /// Sanitizes the input string by cutting up to (and including) the first occurence of a line
    /// terminiating phrase (`\r\n`, `\r`, or `\n`). This is used in sending messages back to
    /// prevent the injection of additional commands.
    pub(crate) fn sanitize(mut data: String) -> String {
        // n.b. ordering matters here to prefer "\r\n" over "\r"
        if let Some((pos, len)) = ["\r\n", "\r", "\n"]
            .iter()
            .flat_map(|needle| data.find(needle).map(|pos| (pos, needle.len())))
            .min_by_key(|&(pos, _)| pos)
        {
            data.truncate(pos + len);
        }
        data
    }
}

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Message>> {
        let result = self.inner.decode(src).map_err(Error::from).and_then(|res| {
            res.map_or(Ok(None), |msg| {
                msg.parse::<Message>().map(Some).map_err(Error::from)
            })
        });
        println!(" << {:?}", result);
        result
    }
}

impl Encoder for IrcCodec {
    type Item = Message;
    type Error = anyhow::Error;

    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> Result<()> {
        println!(">> {:?}", msg);
        self.inner
            .encode(IrcCodec::sanitize(msg.to_string()), dst)
            .map_err(Error::from)
    }
}
