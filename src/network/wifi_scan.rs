use std::{env};
use std::process::{Command};
use serde_derive::{Deserialize, Serialize};
use crate::common::error::Error;

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct Wifi {
    pub mac: String,
    pub ssid: String,
    pub channel: String,
    pub signal_level: String,
    pub security: String,
}

pub async fn scan() -> Result<Vec<Wifi>, Error> {
    const PATH_ENV: &'static str = "PATH";
    let path_system = "/usr/sbin:/sbin";
    let path = env::var_os(PATH_ENV).map_or(path_system.to_string(), |v| {
        format!("{}:{}", v.to_string_lossy().into_owned(), path_system)
    });

    let output = Command::new("iw")
        .env(PATH_ENV, path.clone())
        .arg("dev")
        .output()
        .map_err(|_| Error::CommandNotFound)?;
    let data = String::from_utf8_lossy(&output.stdout);
    let interface = parse_iw_dev(&data)?;

    let output = Command::new("iw")
        .env(PATH_ENV, path)
        .arg("dev")
        .arg(interface)
        .arg("scan")
        .output()
        .map_err(|_| Error::CommandNotFound)?;
    if !output.status.success() {
        return Err(Error::CommandFailed(
            output.status,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let data = String::from_utf8_lossy(&output.stdout);
    parse_iw_dev_scan(&data)
}

fn parse_iw_dev(interfaces: &str) -> Result<String, Error> {
    interfaces
        .split("\tInterface ")
        .take(2)
        .last()
        .ok_or(Error::NoValue)?
        .split("\n")
        .nth(0)
        .ok_or(Error::NoValue)
        .map(|text| text.to_string())
}

fn parse_iw_dev_scan(network_list: &str) -> Result<Vec<Wifi>, Error> {
    let mut wifis: Vec<Wifi> = Vec::new();
    let mut wifi = Wifi::default();
    for line in network_list.split("\n") {
        if let Ok(mac) = extract_value(line, "BSS ", Some("(")) {
            wifi.mac = mac;
        } else if let Ok(signal) = extract_value(line, "\tsignal: ", Some(" dBm")) {
            wifi.signal_level = signal;
        } else if let Ok(channel) = extract_value(line, "\tDS Parameter set: channel ", None) {
            wifi.channel = channel;
        } else if let Ok(ssid) = extract_value(line, "\tSSID: ", None) {
            wifi.ssid = ssid;
        }

        if !wifi.mac.is_empty()
            && !wifi.signal_level.is_empty()
            && !wifi.channel.is_empty()
            && !wifi.ssid.is_empty()
        {
            wifis.push(wifi);
            wifi = Wifi::default();
        }
    }

    Ok(wifis)
}

fn extract_value(line: &str, pattern_start: &str, pattern_end: Option<&str>) -> Result<String, Error> {
    let start = pattern_start.len();
    if start < line.len() && &line[0..start] == pattern_start {
        let end = match pattern_end {
            Some(end) => line.find(end).ok_or(Error::NoValue)?,
            None => line.len(),
        };
        Ok(line[start..end].to_string())
    } else {
        Err(Error::NoValue)
    }
}