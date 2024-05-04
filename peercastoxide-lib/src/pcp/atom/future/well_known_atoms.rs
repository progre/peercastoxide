use crate::pcp::atom::unknown::{well_known_identifiers::*, UnknownAtom};

pub fn pcp_ipv4() -> UnknownAtom {
    UnknownAtom::u32(PCP, 1)
}

pub fn pcp_ipv6() -> UnknownAtom {
    UnknownAtom::u32(PCP, 100)
}

pub fn helo_minimum(session_id: [u8; 16]) -> UnknownAtom {
    UnknownAtom::parent(HELO, vec![UnknownAtom::child(SID, session_id.into())])
}

pub fn oleh(
    session_id: [u8; 16],
    agent_name: &str,
    peer_addr: std::net::SocketAddr,
) -> UnknownAtom {
    UnknownAtom::parent(
        OLEH,
        vec![
            UnknownAtom::child(SID, session_id.into()),
            UnknownAtom::str(AGNT, agent_name).unwrap(),
            UnknownAtom::u32(VER, 1218),
            match peer_addr {
                std::net::SocketAddr::V4(v4) => UnknownAtom::ipv4(RIP, v4.ip()),
                std::net::SocketAddr::V6(v6) => UnknownAtom::ipv6(RIP, v6.ip()),
            },
            UnknownAtom::u16(PORT, peer_addr.port()),
        ],
    )
}
