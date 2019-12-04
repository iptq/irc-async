use anyhow::Result;
use futures::stream::StreamExt;
use futures::future;
use irc_async::{Client, ClientError, Config};

async fn l(mut client: Client) -> Result<()> {
    loop {
        let message = match client.next().await {
            Some(message) => message,
            None => break,
        };
    }
    Ok(())
}

async fn run() -> Result<()> {
    // let config = Config {
    //     host: "chat.freenode.net".into(),
    //     port: 6697,
    //     ssl: true,
    //     nick: "hello".into(),
    // };
    let config = Config {
        host: "127.0.0.1".into(),
        port: 4444,
        ssl: false,
        nick: "hello".into(),
    };
    let (mut client, osu) = Client::with_config(config).await?;
    client.register().await;

    future::join(osu, l(client)).await;

    unreachable!()
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("err: {:?}", err);
    }
}
