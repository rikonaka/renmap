use anyhow::Result;
use pistol::Host;
use pistol::Target;
use pistol::scan::TcpUdpScans;
use pistol::tcp_syn_scan;
use std::net::IpAddr;
use std::time::Duration;

use crate::ScanMode;

fn addr_parser(target_addr: &str) -> Result<Vec<IpAddr>> {
    // 192.168.1.1,192.168.1.2,192.168.1.6-192.168.1.10
    if target_addr.contains(",") {
        let addr_split: Vec<&str> = target_addr.split(",").map(|x| x.trim()).collect();
        let mut ret = Vec::new();
        for s in addr_split {
            if s.contains("-") {
                let s_split: Vec<&str> = s.split("-").map(|x| x.trim()).collect();
                for ss in s_split {
                    let addr: IpAddr = ss.parse()?;
                    ret.push(addr);
                }
            }
        }
        Ok(ret)
    } else {
        let single_addr: IpAddr = target_addr.parse()?;
        Ok(vec![single_addr])
    }
}

fn port_parser(target_port: &str) -> Result<Vec<u16>> {
    // 80,80,100-200
    if target_port.contains(",") {
        let port_split: Vec<&str> = target_port.split(",").map(|x| x.trim()).collect();
        let mut ret = Vec::new();
        for p in port_split {
            if p.contains("-") {
                let s_split: Vec<&str> = p.split("-").map(|x| x.trim()).collect();
                for ss in s_split {
                    let addr: u16 = ss.parse()?;
                    ret.push(addr);
                }
            }
        }
        Ok(ret)
    } else {
        let single_port: u16 = target_port.parse()?;
        Ok(vec![single_port])
    }
}

pub fn pistol_scan(target_addr: &str, target_port: &str, _mode: &ScanMode) -> Result<TcpUdpScans> {
    let target_addr = addr_parser(target_addr)?;
    let target_port = port_parser(target_port)?;
    let mut addrs = Vec::new();
    for a in target_addr {
        let host = Host::new(a, Some(target_port.clone()));
        addrs.push(host);
    }
    let target = Target::new(addrs);
    let timeout = Some(Duration::new(0, 5));
    let tests = 2;
    let ret = tcp_syn_scan(&target, Some(8), None, None, timeout, tests)?;
    Ok(ret)
}
