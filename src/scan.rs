use anyhow::Result;
use pistol::Host;
use pistol::Target;
use pistol::scan::TcpUdpScans;
use pistol::tcp_syn_scan;
use std::net::IpAddr;
use std::time::Duration;

use crate::ScanMode;

pub fn pistol_scan(target_addr: &str, target_port: &str, mode: &ScanMode) -> Result<TcpUdpScans> {
    println!("start scan");
    let target_addr: IpAddr = target_addr.parse()?;
    let port: u16 = target_port.parse()?;
    let host = Host::new(target_addr, Some(vec![port]));
    let target = Target::new(vec![host]);
    let timeout = Some(Duration::new(1, 5));
    let tests = 2;
    let ret = tcp_syn_scan(&target, Some(8), None, None, timeout, tests)?;
    println!("end scan");
    Ok(ret)
}
