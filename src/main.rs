use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    // we need to get this number from a config
    let (pubsub_tx, mut pubsub_rx) = mpsc::channel(1024);

    // rewrite as process_messages or something
    tokio::spawn(async move {
        while let Some(message) = pubsub_rx.recv().await {
            // channel is str but it should be our event struct
            println!("GOT = {}", message);
            // obviously we will need a db of users to select from here.
            // we should maintain a hashmap that must not be used from the connector tasks.
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        let pubsub_tx = pubsub_tx.clone();

        // rewrite as socket_handler or something
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    // Return value of `Ok(0)` signifies that the remote has
                    // closed
                    Ok(0) => return,
                    Ok(n) => {
                        // Copy the data back to socket
                        if socket.write_all(&buf[..n]).await.is_err() {
                            // Unexpected socket error. There isn't much we can
                            // do here so just stop processing.
                            return;
                        }
                        if pubsub_tx
                            .send(format!("Hello there, socket said something: {:?}", &buf))
                            .await
                            .is_err()
                        {
                            // Well, let's die.
                            return;
                        }
                    }
                    Err(_) => {
                        // Unexpected socket error. There isn't much we can do
                        // here so just stop processing.
                        return;
                    }
                }
            }
        });
    }
}
