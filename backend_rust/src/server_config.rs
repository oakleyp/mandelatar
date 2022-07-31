use log::error;
use std::env;

const SERVER_PORT_DEFAULT: u16 = 8080;

pub struct ServerConfig {
    pub server_addr: String,
    pub server_port: u16,
}

impl ServerConfig {
    pub fn load_from_env() -> Self {
        Self {
            server_addr: match env::var("MANDELATAR_SERVER_ADDR") {
                Ok(host) => host,
                Err(_) => "127.0.0.1".to_string(),
            },
            server_port: match env::var("MANDELATAR_SERVER_PORT") {
                Ok(port) => u16::from_str_radix(port.as_str(), 10).unwrap_or_else(|e| {
                    error!(
                        "Failed to parse port - falling back to default {} - {}",
                        SERVER_PORT_DEFAULT, e
                    );
                    SERVER_PORT_DEFAULT
                }),
                Err(_) => SERVER_PORT_DEFAULT,
            },
        }
    }
}
