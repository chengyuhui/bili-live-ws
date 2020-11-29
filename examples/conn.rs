use bili_live_ws::packet::ClientAuth;
use bili_live_ws::{
    header::{Header, PacketType, PacketVer},
    packet::{parse_packet, Packet, HEARTBEAT_CONTENT},
};
use futures_util::{SinkExt, StreamExt};
use serde_json::to_vec;
use std::{error::Error, time::Duration};
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (servers, token) = bili_live_ws::get_servers("1022")?;
    dbg!(&servers, &token);
    let server = &servers[0];
    let url = format!("wss://{}/sub", server.host);

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut auth_pkt_content = to_vec(&ClientAuth::new(1022, &token))?;
    let mut auth_pkt_header = Header::new(
        auth_pkt_content.len(),
        PacketType::ClientAuth,
        PacketVer::Plain,
    )
    .to_vec()?;
    auth_pkt_header.append(&mut auth_pkt_content);

    write.send(Message::binary(auth_pkt_header)).await?;

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            let res = write
                .send(Message::binary(Vec::from(HEARTBEAT_CONTENT)))
                .await;
            if res.is_err() {
                break;
            }
            interval.tick().await;
        }
    });

    read.for_each(|msg| async {
        if let Ok(Message::Binary(msg)) = msg {
            for pkt in parse_packet(&msg) {
                if let Some(Packet::Danmu(msg)) = pkt {
                    if let Some(ref medal) = msg.medal {
                        println!("[{}] {}: {}", medal, msg.user, msg.text);
                    } else {
                        println!("{}: {}", msg.user, msg.text);
                    }
                } else {
                    println!("{:?}", pkt);
                }
            }
        }
    })
    .await;

    Ok(())
}
