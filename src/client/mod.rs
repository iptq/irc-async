mod config;
mod stream;

use std::io;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use anyhow::Result;
use futures::future::{self, Either, Future, FutureExt, Ready};
use futures::channel::mpsc::{self, UnboundedSender};
use futures::sink::SinkExt;
use futures::stream::{FilterMap, SplitSink, SplitStream, Stream, StreamExt};
use native_tls::TlsConnector;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_tls::{TlsConnector as TokioTlsConnector, TlsStream};
use tokio_util::codec::{Decoder, Framed};

use crate::client::stream::ClientStream;
use crate::proto::{Command, IrcCodec, Message};

pub use self::config::Config;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("fuck")]
    Io(#[from] io::Error),
    #[error("fuck")]
    Tls(#[from] native_tls::Error),
}

pub struct Client {
    config: Config,
    stream: Pin<Box<dyn Stream<Item = Result<Message>>>>,
    tx: UnboundedSender<Message>,
}

pub type Osu = Pin<Box<dyn Future<Output=Result<()>> + Send>>;

impl Client {
    pub async fn with_config(config: Config) -> Result<(Self, Osu)> {
        let mut addrs = (config.host.as_ref(), config.port).to_socket_addrs()?;
        let mut stream = TcpStream::connect(addrs.next().unwrap()).await?;

        let stream = if config.ssl {
            let connector: TokioTlsConnector = TlsConnector::new().unwrap().into();
            let stream = connector.connect(&config.host, stream).await?;
            ClientStream::Tls(stream)
        } else {
            ClientStream::Plain(stream)
        };

        let stream = IrcCodec::new().framed(stream);
        let (sink, stream) = stream.split();
        let (tx, filter_rx) = mpsc::unbounded();
        let mut filter_tx = tx.clone();

        let stream = stream.filter_map(move |message| {
            if let Ok(Message {
                command: Command::PING(code, _),
                ..
            }) = message
            {
                let mut filter_tx = filter_tx.clone();
                Either::Left(async move {
                    filter_tx.send(Message {
                        tags: None,
                        prefix: None,
                        command: Command::PONG(code, None),
                    }).await;
                    None
                })
            } else {
                Either::Right(future::ready(Some(message)))
            }
        });

        let osu = filter_rx.map(Ok).forward(sink).boxed();

        let client = Client {
            config,
            stream: stream.boxed(),
            tx,
        };
        Ok((client, osu))
    }

    pub async fn register(&mut self) -> Result<()> {
        self.send(Message {
            tags: None,
            prefix: None,
            command: Command::NICK(self.config.nick.clone()),
        })
        .await?;
        self.send(Message {
            tags: None,
            prefix: None,
            command: Command::USER(
                self.config.nick.clone(),
                self.config.nick.clone(),
                self.config.nick.clone(),
            ),
        })
        .await
    }

    pub async fn send(&mut self, message: Message) -> Result<()> {
        self.tx.send(message).await?;
        self.tx.flush().await?;
        Ok(())
    }
}

impl Stream for Client {
    type Item = Result<Message>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.get_mut().stream), context)
    }
}
