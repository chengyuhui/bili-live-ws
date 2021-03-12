use bilibili_live_danmaku::{connect, packet::Packet};

use colored::Colorize;
use futures_util::{future, StreamExt};
use std::error::Error;

mod get_servers;

fn print_medal(name: &str, level: u64) {
    print!("[{} {}] ", name.green(), level.to_string().red());
}

fn print_packet(pkt: Packet) {
    match pkt {
        Packet::Danmu(dm) => {
            if let Some(mi) = dm.medal {
                print_medal(&mi.name, mi.level);
            }
            println!("{}: {}", dm.user.name.bright_blue().bold(), dm.text);
        }
        Packet::Gift(gift) => {
            println!(
                "{} {} {} * {}",
                gift.uname.bright_blue().bold(),
                gift.action,
                gift.gift_name.red().bold(),
                gift.num
            );
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let room_id: u64 = args[1].parse()?;
    let (servers, token) = get_servers::get_servers(room_id).await?;

    let server = &servers[0];

    let stream = connect(&server.host, room_id, Some(&token)).await?;

    stream
        .for_each(|pkts| {
            if let Ok(pkts) = pkts {
                for pkt in pkts {
                    print_packet(pkt);
                }
            }
            future::ready(())
        })
        .await;

    Ok(())
}
