use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::slice::from_raw_parts;
use libc::{if_nametoindex, sockaddr_in, sockaddr_in6, sockaddr_ll, strlen, AF_INET, AF_INET6, AF_PACKET};
use crate::common::error::Error;
use crate::common::unix::{ipv4_from_in_addr, ipv6_from_in6_addr, make_ipv4_netmask, make_ipv6_netmask};
use crate::models::interface::NetworkInterface;
use crate::network::getifaddrs::getifaddrs;


pub fn get_interfaces() -> Result<Vec<NetworkInterface>, Error> {
    let mut network_interfaces: HashMap<String, NetworkInterface> = HashMap::new();

    for netifa in getifaddrs()? {
        let netifa_addr = netifa.ifa_addr;
        let netifa_family = if netifa_addr.is_null() {
            continue;
        } else {
            unsafe { (*netifa_addr).sa_family as i32 }
        };

        let mut network_interface = match netifa_family {
            AF_PACKET => {
                let name = make_netifa_name(&netifa)?;
                let mac = make_mac_addrs(&netifa);
                let index = netifa_index(&netifa);
                NetworkInterface {
                    name,
                    addr: Vec::new(),
                    mac_addr: Some(mac),
                    index,
                }
            }
            AF_INET => {
                let socket_addr = netifa_addr as *mut sockaddr_in;
                let internet_address = unsafe { (*socket_addr).sin_addr };
                let name = make_netifa_name(&netifa)?;
                let index = netifa_index(&netifa);
                let netmask = make_ipv4_netmask(&netifa);
                let addr = ipv4_from_in_addr(&internet_address)?;
                let broadcast = make_ipv4_broadcast_addr(&netifa)?;
                NetworkInterface::new_afinet(name.as_str(), addr, netmask, broadcast, index)
            }
            AF_INET6 => {
                let socket_addr = netifa_addr as *mut sockaddr_in6;
                let internet_address = unsafe { (*socket_addr).sin6_addr };
                let name = make_netifa_name(&netifa)?;
                let index = netifa_index(&netifa);
                let netmask = make_ipv6_netmask(&netifa);
                let addr = ipv6_from_in6_addr(&internet_address)?;
                let broadcast = make_ipv6_broadcast_addr(&netifa)?;
                NetworkInterface::new_afinet6(name.as_str(), addr, netmask, broadcast, index)
            }
            _ => continue,
        };

        network_interfaces
            .entry(network_interface.name.clone())
            .and_modify(|old| old.addr.append(&mut network_interface.addr))
            .or_insert(network_interface);
    }

    Ok(network_interfaces.into_values().collect())
}


fn make_netifa_name(netifa: &libc::ifaddrs) -> Result<String, Error> {
    let data = netifa.ifa_name as *const libc::c_char;
    let len = unsafe { strlen(data) };
    let bytes_slice = unsafe { from_raw_parts(data as *const u8, len) };

    match String::from_utf8(bytes_slice.to_vec()) {
        Ok(s) => Ok(s),
        Err(e) => Err(Error::ParseUtf8Error(e)),
    }
}

fn make_ipv4_broadcast_addr(netifa: &libc::ifaddrs) -> Result<Option<Ipv4Addr>, Error> {
    let ifa_dstaddr = netifa.ifa_ifu;

    if ifa_dstaddr.is_null() {
        return Ok(None);
    }

    let socket_addr = ifa_dstaddr as *mut sockaddr_in;
    let internet_address = unsafe { (*socket_addr).sin_addr };
    let addr = ipv4_from_in_addr(&internet_address)?;

    Ok(Some(addr))
}

fn make_ipv6_broadcast_addr(netifa: &libc::ifaddrs) -> Result<Option<Ipv6Addr>, Error> {
    let ifa_dstaddr = netifa.ifa_ifu;

    if ifa_dstaddr.is_null() {
        return Ok(None);
    }

    let socket_addr = ifa_dstaddr as *mut sockaddr_in6;
    let internet_address = unsafe { (*socket_addr).sin6_addr };
    let addr = ipv6_from_in6_addr(&internet_address)?;

    Ok(Some(addr))
}

fn make_mac_addrs(netifa: &libc::ifaddrs) -> String {
    let netifa_addr = netifa.ifa_addr;
    let socket_addr = netifa_addr as *mut sockaddr_ll;
    let mac_array = unsafe { (*socket_addr).sll_addr };
    let addr_len = unsafe { (*socket_addr).sll_halen };
    let real_addr_len = std::cmp::min(addr_len as usize, mac_array.len());
    let mac_slice = unsafe { from_raw_parts(mac_array.as_ptr(), real_addr_len) };

    mac_slice
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<Vec<_>>()
        .join(":")
}

fn netifa_index(netifa: &libc::ifaddrs) -> u32 {
    let name = netifa.ifa_name as *const libc::c_char;

    unsafe { if_nametoindex(name) }
}