mod config;

use std::io;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::sync::mpsc::{self, Receiver, Sender};
use std::task::{Context, Poll};

use futures_util::stream::{SplitSink, SplitStream};
use native_tls::TlsConnector;
use tokio::codec::{Decoder, Framed};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::stream::Stream;
use tokio_tls::{TlsConnector as TokioTlsConnector, TlsStream};

use crate::proto::{CodecError, Command, IrcCodec, Message};

pub use self::config::Config;

#[derive(Debug)]
pub enum ClientError {
    Io(io::Error),
    Tls(native_tls::Error),
}

impl From<io::Error> for ClientError {
    fn from(err: io::Error) -> Self {
        ClientError::Io(err)
    }
}

impl From<native_tls::Error> for ClientError {
    fn from(err: native_tls::Error) -> Self {
        ClientError::Tls(err)
    }
}

pub struct Client {
    config: Config,
    stream: SplitStream<Framed<ClientStream, IrcCodec>>,
    sink: SplitSink<Framed<ClientStream, IrcCodec>, Message>,
    filter_rx: Receiver<Message>,
}

impl Client {
    pub async fn with_config(config: Config) -> Result<Self, ClientError> {
        let mut addrs = (config.host.as_ref(), config.port).to_socket_addrs()?;
        let mut stream = TcpStream::connect(addrs.next().unwrap()).await?;

        let stream = if config.ssl {
            let connector: TokioTlsConnector = TlsConnector::new().unwrap().into();
            let stream = connector.connect(&config.host, stream).await?;
            ClientStream::Tls(stream)
        } else {
            ClientStream::Plain(stream)
        };
        let stream = IrcCodec.framed(stream);
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
            prefix: None,
            command: Command::Nick(self.config.nick.clone()),
        })
        .await;
        self.send(Message {
            prefix: None,
            command: Command::User(
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
    type Item = Result<Message, CodecError>;

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
