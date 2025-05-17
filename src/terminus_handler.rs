// A Terminus is responsible for tracking the state of a particular stream of incoming or outgoing messages.
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::amqp::types::frame::{Frame, FrameType};

// Sources track outgoing messages.
// Messages may only travel along a Link if they meet the entry criteria at the Source.
// TODO: logging
pub async fn source_handler<W>(mut socket_writer: W, mut client_rx: Receiver<Frame>)
where
    W: AsyncWriteExt + Unpin + Send + 'static,
{
    let mut buf = vec![0; 8192];
    let open_frame = Frame::open();
    socket_writer.write_buf(&mut open_frame.as_bytes());

    loop {
        match client_rx.recv().await {
            Some(frame) => {
                match frame.frame_type {
                    FrameType::AMQP => {}
                    _ => {
                        continue;
                    }
                }
                socket_writer.write_buf(&mut frame.as_bytes());
            }
            None => {
                break;
            }
        }
    }
}

// Targets track incoming messages.
// TODO: logging
pub async fn target_handler<R>(mut socket_reader: R, frame_bus_tx: Sender<Frame>)
where
    R: AsyncReadExt + Unpin + Send + 'static,
{
    loop {
        let mut buf = [0u8; 8192];
        match socket_reader.read(&mut buf).await {
            // closed connection
            Ok(0) => {
                break;
            }
            Ok(_) => {
                let frame = Frame::new(&mut socket_reader).await;
                match frame {
                    Ok(frame) => match frame_bus_tx.send(frame).await {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }
    let close_frame = Frame::close();
    frame_bus_tx.send(close_frame).await.unwrap_or(());
}
