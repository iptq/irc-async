use anyhow::Result;
use futures::future;
use futures::stream::StreamExt;
use irc_async::{Client, ClientError, Config};

async fn run() -> Result<()> {
    let config = Config {
        host: "127.0.0.1".into(),
        port: 4444,
        ssl: false,
        nick: "hello".into(),
    };
    let (mut client, osu) = Client::with_config(config).await?;
    client.register().await;

    let handler = async {
        while let Some(Ok(message)) = client.next().await {
            println!("message: {:?}", message);
            client.send(message);
        }
    };

    future::join(osu, handler).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("err: {:?}", err);
    }
}
