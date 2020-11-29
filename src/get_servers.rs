use crate::{Error, Result};
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

pub fn get_servers(room: &str) -> Result<(Vec<Server>, String)> {
    let url = "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?&type=0";
    let resp = attohttpc::get(url)
        .param("id", room)
        .header("User-Agent", USER_AGENT)
        .send()?
        .error_for_status()?;
    let resp: GetServerResponse = resp.json()?;

    let servers = resp.data.host_list;
    if servers.is_empty() {
        Err(Error::NoServer)
    } else {
        Ok((servers, resp.data.token))
    }
}
