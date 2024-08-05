use std::{error::Error, fs, net::Ipv4Addr, str::FromStr};

const FILENAME: &str = "server.cfg";

#[derive(Debug)]
pub struct Config {
    pub host: Ipv4Addr,
    pub port: u16,
}

impl Config {
    pub fn build() -> Result<Config, Box<dyn Error>> {
        let config_data = fs::read_to_string(FILENAME)?;

        let lines: Vec<&str> = config_data.lines().collect();

        if lines.len() < 2 {
            Err("Not enough data to construct the Config")?;
        }

        let host = Ipv4Addr::from_str(lines[0].trim())?;
        let port = lines[1].parse::<u16>()?;

        Ok(Config { 
            host: host,
            port: port, 
        })
    }
}
