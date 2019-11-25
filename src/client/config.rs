use std::net::SocketAddr;

use derive_builder::Builder;

pub struct Config {
    pub host: String,
    pub port: u16,
    pub ssl: bool,
    pub nick: String,
}
