use std::net::SocketAddr;

use super::{Atom, *};

pub fn pcp_ipv4() -> Atom {
    Atom::u32(PCP, 1)
}

pub fn pcp_ipv6() -> Atom {
    Atom::u32(PCP, 100)
}

pub fn helo_minimum(session_id: [u8; 16]) -> Atom {
    Atom::parent(HELO, vec![Atom::child(SID, session_id.into())])
}

pub fn oleh(session_id: [u8; 16], agent_name: &str, peer_addr: SocketAddr) -> Atom {
    Atom::parent(
        OLEH,
        vec![
            Atom::child(SID, session_id.into()),
            Atom::str(AGNT, agent_name).unwrap(),
            Atom::u32(VER, 1218),
            match peer_addr {
                SocketAddr::V4(v4) => Atom::ipv4(RIP, v4.ip()),
                SocketAddr::V6(v6) => Atom::ipv6(RIP, v6.ip()),
            },
            Atom::u16(PORT, peer_addr.port()),
        ],
    )
}

pub fn oleh_minimum(session_id: Vec<u8>) -> Atom {
    Atom::parent(OLEH, vec![Atom::child(SID, session_id)])
}
