mod config;

use std::io;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::sync::mpsc::{self, Receiver, Sender};
use std::task::{Context, Poll};

use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::{SplitSink, SplitStream, Stream, StreamExt};
use native_tls::TlsConnector;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_tls::{TlsConnector as TokioTlsConnector, TlsStream};
use tokio_util::codec::{Decoder, Framed};

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
    stream: SplitStream<Framed<ClientStream, IrcCodec>>,
    sink: SplitSink<Framed<ClientStream, IrcCodec>, Message>,
    filter_rx: Receiver<Message>,
}

impl Client {
    pub async fn with_config(config: Config) -> Result<Self> {
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
        let (filter_tx, filter_rx) = mpsc::channel();
        let client = Client {
            config,
            stream,
            sink,
            filter_rx,
        };
        Ok(client)
    }

    pub async fn register(&mut self) {
        self.send(Message {
            tags: None,
            prefix: None,
            command: Command::NICK(self.config.nick.clone()),
        })
        .await;
        self.send(Message {
            tags: None,
            prefix: None,
            command: Command::USER(
                self.config.nick.clone(),
                self.config.nick.clone(),
                self.config.nick.clone(),
            ),
        })
        .await;
    }

    pub async fn send(&mut self, message: Message) {
        self.sink.send(message).await;
        self.sink.flush().await;
    }
}

impl Stream for Client {
    type Item = Result<Message>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.get_mut().stream), context)
    }
}

enum ClientStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for ClientStream {
    fn poll_read(
        self: Pin<&mut Self>,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_read(Pin::new(stream), context, buf),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_read(Pin::new(stream), context, buf)
            }
        }
    }
}

impl AsyncWrite for ClientStream {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_write(Pin::new(stream), context, buf),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_write(Pin::new(stream), context, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_flush(Pin::new(stream), context),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_flush(Pin::new(stream), context)
            }
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_shutdown(Pin::new(stream), context),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_shutdown(Pin::new(stream), context)
            }
        }
    }
}
