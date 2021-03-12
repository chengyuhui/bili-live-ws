mod error;
pub mod header;
pub mod packet;

use std::{pin::Pin, task::Poll, time::Duration};

pub use error::{Error, Result};

use futures_util::{ready, stream::Stream, SinkExt, StreamExt, TryStreamExt};
use header::{Header, PacketType, PacketVer};
use packet::{parse_packet, ClientAuth, Packets, HEARTBEAT_CONTENT};
use serde_json::to_vec;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn connect(
    host: &str,
    room_id: u64,
    token: Option<&str>,
) -> Result<impl Stream<Item = Result<Packets>>> {
    let url = format!("wss://{}/sub", host);
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, read) = ws_stream.split();

    // Auth
    let mut auth_pkt_content = to_vec(&ClientAuth::new(room_id, token)).unwrap();
    let mut auth_pkt_header = Header::new(
        auth_pkt_content.len(),
        PacketType::ClientAuth,
        PacketVer::Plain,
    )
    .to_vec()?;
    auth_pkt_header.append(&mut auth_pkt_content);

    write.send(Message::binary(auth_pkt_header)).await?;

    // Auth done
    let (close_ch_s, mut close_ch_r) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            // Stop if the channel is closed or sent
            match close_ch_r.try_recv() {
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                _ => break,
            }

            let res = write
                .send(Message::binary(Vec::from(HEARTBEAT_CONTENT)))
                .await;
            if res.is_err() {
                break;
            }

            interval.tick().await;
        }
    });

    Ok(DanmakuStream {
        inner: read.err_into(),
        _close_ch_s: close_ch_s,
    })
}

struct DanmakuStream<S> {
    inner: S,
    _close_ch_s: tokio::sync::oneshot::Sender<()>,
}

impl<S: Stream<Item = Result<Message>> + Unpin> Stream for DanmakuStream<S> {
    type Item = Result<Packets>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let p = ready!(Pin::new(&mut self.inner).poll_next(cx));
        let r = match p {
            Some(Ok(Message::Binary(msg))) => Some(Ok(parse_packet(&msg))),
            Some(Ok(_)) => None,
            Some(Err(e)) => Some(Err(e)),
            None => None,
        };
        Poll::Ready(r)
    }
}
