use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

pub type Netmask<T> = Option<T>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub addr: Vec<Addr>,
    pub mac_addr: Option<String>,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Addr {
    V4(V4IfAddr),
    V6(V6IfAddr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct V4IfAddr {
    pub ip: Ipv4Addr,
    pub broadcast: Option<Ipv4Addr>,
    pub netmask: Netmask<Ipv4Addr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct V6IfAddr {
    pub ip: Ipv6Addr,
    pub broadcast: Option<Ipv6Addr>,
    pub netmask: Netmask<Ipv6Addr>,
}

impl NetworkInterface {
    pub fn new_afinet(
        name: &str,
        addr: Ipv4Addr,
        netmask: Netmask<Ipv4Addr>,
        broadcast: Option<Ipv4Addr>,
        index: u32,
    ) -> NetworkInterface {
        let ifaddr_v4 = V4IfAddr {
            ip: addr,
            broadcast,
            netmask,
        };

        NetworkInterface {
            name: name.to_string(),
            addr: vec![Addr::V4(ifaddr_v4)],
            mac_addr: None,
            index,
        }
    }

    pub fn new_afinet6(
        name: &str,
        addr: Ipv6Addr,
        netmask: Netmask<Ipv6Addr>,
        broadcast: Option<Ipv6Addr>,
        index: u32,
    ) -> NetworkInterface {
        let ifaddr_v6 = V6IfAddr {
            ip: addr,
            broadcast,
            netmask,
        };

        NetworkInterface {
            name: name.to_string(),
            addr: vec![Addr::V6(ifaddr_v6)],
            mac_addr: None,
            index,
        }
    }
}