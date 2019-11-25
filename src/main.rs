#![feature(never_type)]

mod client;
mod proto;

use std::convert::TryInto;
use std::net::ToSocketAddrs;

use rustyline::{error::ReadlineError, Editor};
use tokio::prelude::*;

use crate::client::{Client, ClientError, Config};

async fn run() -> Result<!, ClientError> {
    let config = Config {
        host: "chat.freenode.net".into(),
        port: 6697,
        ssl: true,
        nick: "irc-async".into(),
    };
    let mut client = Client::with_config(config).await?;
    client.register().await;

    loop {
        let message = match client.next().await {
            Some(message) => message,
            None => break,
        };
    }

    unreachable!()
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("err: {:?}", err);
    }
}
