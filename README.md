```rs
async fn run() -> Result<(), ClientError> {
    let config = Config {
        host: "chat.freenode.net".into(),
        port: 6697,
        ssl: true,
        nick: "hello".into(),
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
```
