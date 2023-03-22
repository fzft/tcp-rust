#![allow(unused)]

use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use tun_tap::{Iface, Mode};

mod tcp;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = HashMap::new();
    let mut nic = Iface::new("tun0", Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let bytes_read = nic.recv(&mut buf[..])?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let proto = u16::from_be_bytes([buf[2], buf[3]]);
        // 0x086dd is ipv6
        if proto != 0x0800 {
            // no ipv4
            continue;
        }

        match Ipv4HeaderSlice::from_slice(&buf[4..bytes_read]) {
            Ok(iph) => {

                let src = iph.source_addr();
                let dst = iph.destination_addr();
                let proto = iph.protocol();
                if proto != 0x06 {
                    continue;
                }

                let ip_hdr_sz = iph.slice().len();
                match TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..bytes_read]) {
                    Ok(tcph) => {
                        let data = 4 + ip_hdr_sz + tcph.slice().len();
                        connections.entry(Quad {
                            src: (src, tcph.source_port()),
                            dst: (dst, tcph.destination_port()),
                        }).or_default().on_packet(&mut nic, iph, tcph, &buf[data..bytes_read]);

                    }
                    Err(e) => {}
                }
            }
            Err(e) => {
                eprintln!("ignoring weird packet {:?}", e)
            }
        }
    }
    Ok(())
}

