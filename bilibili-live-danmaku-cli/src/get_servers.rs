use anyhow::Result;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct GetServerResponse {
    code: i64,
    ttl: i64,
    data: GetServerData,
}

#[derive(Debug, Deserialize)]
struct GetServerData {
    host_list: Vec<Server>,
    max_delay: u64,
    refresh_rate: u64,
    refresh_row_factor: f64,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub ws_port: u16,
    pub wss_port: u16,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36";

pub async fn get_servers(room: u64) -> Result<(Vec<Server>, String)> {
    let url = "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?type=0";

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&[("id", room)])
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?;
    let resp: GetServerResponse = resp.json().await?;

    Ok((resp.data.host_list, resp.data.token))
}
