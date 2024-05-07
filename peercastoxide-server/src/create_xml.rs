use std::{net::SocketAddr, time::Instant};

use peercastoxide_lib::{
    pcp::atom::well_known_atoms::Bcst,
    peercast_xml::{
        self, Bandwidth, Channel, ChannelsFound, ChannelsRelayed, Connections, Hits, Host, Servent,
        Track,
    },
};

pub struct Record {
    pub bcst: Bcst,
    pub created_at: Instant,
    pub updated_at: Instant,
}

fn to_channel(record: &Record) -> Channel {
    let bcst = &record.bcst;
    let host = &bcst.host;
    let flg1 = &host.flg1;
    let chan = &bcst.chan;
    let info = &chan.info;
    let trck = &chan.trck;
    let hosts = vec![Host {
        ip: host
            .ip_port
            .first()
            .map(|(ip, port)| SocketAddr::new(ip.0, *port)),
        hops: bcst.hops,
        listeners: host.numl,
        relays: host.numr,
        uptime: host.uptm,
        push: flg1.push(),
        relay: flg1.relay(),
        direct: flg1.direct(),
        cin: flg1.cin(),
        stable: 0,
        version: host.ver,
        update: record.updated_at.elapsed().as_secs(),
        tracker: flg1.tracker(),
    }];
    Channel {
        name: info.name.clone(),
        id: chan.id.to_string(),
        bitrate: info.bitr.unwrap_or_default(),
        r#type: info.r#type.clone().unwrap_or_default(),
        genre: info.gnre.clone(),
        desc: info.desc.clone(),
        url: info.url.clone(),
        uptime: 0,
        comment: info.cmnt.clone(),
        age: record.created_at.elapsed().as_secs(),
        skips: Some(0),
        skip: None,
        bcflags: 0,
        hits: Hits {
            hosts: 1,
            listeners: hosts.iter().map(|x| x.listeners).sum(),
            relays: hosts.iter().map(|x| x.relays).sum(),
            firewalled: {
                host.ip_port
                    .first()
                    .map(|&(_, port)| port == 0)
                    .unwrap_or(true)
            },
            closest: host.oldp.unwrap_or_default(),
            furthest: bcst.hops,
            newest: host.newp.unwrap_or_default(),
            host: hosts,
        },
        relay: None,
        track: Track {
            title: trck.titl.clone(),
            artist: trck.crea.clone(),
            album: trck.albm.clone(),
            genre: trck.gnre.clone().unwrap_or_default(),
            contact: trck.url.clone(),
        },
    }
}

pub fn create_xml(total_connections: u32, server_start_time: Instant, db: &[&Record]) -> String {
    let uptime = server_start_time.elapsed().as_secs();
    let xml = peercast_xml::Peercast {
        session: None,
        servent: Servent { uptime },
        bandwidth: Bandwidth { r#in: 0, out: 0 },
        connections: Connections {
            total: total_connections,
            relays: 0,
            direct: 0,
        },
        channels_relayed: ChannelsRelayed {
            total: 0,
            channel: vec![],
        },
        channels_found: ChannelsFound {
            total: db.len() as u32,
            channel: db.iter().map(|x| to_channel(x)).collect(),
        },
        host_cache: None,
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\" ?>\n{}",
        quick_xml::se::to_string_with_root("peercast", &xml).unwrap()
    )
}
