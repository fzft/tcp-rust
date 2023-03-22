use std::io;

use etherparse::{Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    EStab,

}

pub struct Connection {
    state: State,
    send: SendSequence,
    recv: RecvSequence,
}

struct SendSequence {
    // send unacknowledged
    una: usize,
    // send next
    nxt: usize,
    // send window
    wnd: usize,
    // send urgent pointer
    up: usize,
    wl1: usize,
    wl2: usize,
    iss: usize,
}

struct RecvSequence {
    // receive next
    nxt: usize,
    // receive window
    wnd: usize,
    // receive urgent pointer
    up: usize,
    irs: usize,
}


impl Default for Connection {
    fn default() -> Self {
        // Self :: Closed
        Self {
            state: State::Listen,
        }
    }
}

impl Connection {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: Ipv4HeaderSlice<'a>,
        tcph: TcpHeaderSlice<'a>,
        data: &'a [u8]) -> io::Result<(usize)> {
        let mut buf = [0u8; 1504];
        match self.state {
            State::Closed => {
                return Ok(0);
            }
            State::Listen => {
                if !tcph.syn() {
                    // only expected syn
                    return Ok(0);
                }

                // need to start establishing connection
                let mut syn_ack = TcpHeader::new(
                    tcph.destination_port(),
                    tcph.source_port(),
                    0,
                    0);
                syn_ack.syn = true;
                syn_ack.ack = true;

                let mut ip = Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    0x06,
                    iph.destination(),
                    iph.source(),
                );
                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };
                nic.send(&buf[..unwritten])
            }
            _ => {
                Ok(0)
            }
        }


        // println!("{} - {} {}b of tcp to port {} ",
        //          iph.source_addr(),
        //          iph.destination_addr(),
        //          tcph.slice().len(),
        //          tcph.destination_port());
    }
}

