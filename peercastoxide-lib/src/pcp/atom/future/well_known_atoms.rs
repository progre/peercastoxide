use super::{custom_atom::CustomAtom, well_known_identifiers::*};

pub fn pcp_ipv4() -> CustomAtom {
    CustomAtom::u32(PCP, 1)
}

pub fn pcp_ipv6() -> CustomAtom {
    CustomAtom::u32(PCP, 100)
}

pub fn helo_minimum(session_id: [u8; 16]) -> CustomAtom {
    CustomAtom::parent(HELO, vec![CustomAtom::child(SID, session_id.into())])
}

pub fn oleh(session_id: [u8; 16], agent_name: &str, peer_addr: std::net::SocketAddr) -> CustomAtom {
    CustomAtom::parent(
        OLEH,
        vec![
            CustomAtom::child(SID, session_id.into()),
            CustomAtom::str(AGNT, agent_name).unwrap(),
            CustomAtom::u32(VER, 1218),
            match peer_addr {
                std::net::SocketAddr::V4(v4) => CustomAtom::ipv4(RIP, v4.ip()),
                std::net::SocketAddr::V6(v6) => CustomAtom::ipv6(RIP, v6.ip()),
            },
            CustomAtom::u16(PORT, peer_addr.port()),
        ],
    )
}
