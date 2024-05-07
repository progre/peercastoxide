use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use anyhow::Result;
use http_body_util::Full;
use hyper::{body::Bytes, header::CONTENT_TYPE, server::conn::http1, Method, Response, StatusCode};
use hyper_util::rt::TokioIo;
use peercastoxide_lib::pcp::atom::{
    from_unknown,
    values::Id,
    well_known_atoms::{Bcst, Quit},
    well_known_identifiers::{BCST, QUIT},
    well_known_protocols::handshake,
    AtomStreamReader, AtomStreamWriter,
};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    select, spawn,
    time::timeout,
};
use tracing::error;

use crate::create_xml::{create_xml, Record};

pub type Db = (u32, HashMap<Id, Record>);

const AGENT_NAME: &str = concat!("PeerCastOxide/", env!("CARGO_PKG_VERSION"));

async fn process_pcp(stream: TcpStream, db: Arc<RwLock<Db>>) -> Result<()> {
    let session_id = Id(rand::random());
    let peer_addr = stream.peer_addr()?;
    let (reader, writer) = stream.into_split();
    let mut reader = AtomStreamReader::new(reader);
    let mut writer = AtomStreamWriter::new(writer);

    timeout(
        Duration::from_secs(15),
        handshake(
            &mut reader,
            &mut writer,
            &session_id,
            peer_addr.ip(),
            AGENT_NAME,
            Duration::from_secs(5),
        ),
    )
    .await??;

    loop {
        let atom = reader.read_unknown_atom().await?;
        match atom.identifier().0.as_ref() {
            BCST => {
                let bcst: Bcst = from_unknown(atom)?;
                let mut db = db.write().unwrap();
                tracing::trace!("{:#?}", bcst);
                if let Some(record) = db.1.get_mut(&bcst.chan.id) {
                    record.bcst = bcst;
                    record.updated_at = Instant::now();
                    continue;
                }
                db.1.insert(
                    bcst.chan.id.clone(),
                    Record {
                        bcst,
                        created_at: Instant::now(),
                        updated_at: Instant::now(),
                    },
                );
            }
            QUIT => {
                let quit: Quit = from_unknown(atom)?;
                tracing::trace!("{:#?}", quit);
            }
            _ => {
                tracing::trace!("{:#?}", atom);
            }
        }
    }
}

async fn process_http(
    stream: TcpStream,
    server_start_time: Instant,
    db: Arc<RwLock<Db>>,
) -> Result<()> {
    let io = TokioIo::new(stream);

    let service = hyper::service::service_fn(|req| {
        let db = db.clone();
        async move {
            if req.method() != Method::GET {
                return Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new(Bytes::from(
                        StatusCode::METHOD_NOT_ALLOWED.as_str(),
                    )))
                    .unwrap());
            }
            if req.uri() != "/admin?cmd=viewxml" {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(CONTENT_TYPE, "text/plain")
                    .body(Full::new(Bytes::from(
                        StatusCode::NOT_FOUND.canonical_reason().unwrap_or_default(),
                    )))
                    .unwrap());
            }
            let xml = {
                let db = db.read().unwrap();
                create_xml(db.0, server_start_time, &db.1.values().collect::<Vec<_>>())
            };
            Response::builder()
                .header(CONTENT_TYPE, "application/xml")
                .body(Full::new(Bytes::from(xml)))
        }
    });

    http1::Builder::new().serve_connection(io, service).await?;
    Ok(())
}

async fn process(stream: TcpStream, server_start_time: Instant, db: Arc<RwLock<Db>>) -> Result<()> {
    let mut buf = [0u8; 4];
    stream.peek(&mut buf).await?;
    if &buf == b"pcp\n" {
        struct ScopeExit(Arc<RwLock<Db>>);
        impl Drop for ScopeExit {
            fn drop(&mut self) {
                self.0.write().unwrap().0 -= 1;
            }
        }
        let _scope = ScopeExit(db.clone());
        db.write().unwrap().0 += 1;
        process_pcp(stream, db).await
    } else {
        process_http(stream, server_start_time, db).await
    }
}

async fn accept_connenctions_loop(
    addr: impl ToSocketAddrs,
    server_start_time: Instant,
    db: Arc<RwLock<Db>>,
) -> Result<()> {
    let listener = TcpListener::bind(&addr).await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tracing::trace!("accept: {}", socket.peer_addr()?);
        let db = db.clone();
        spawn(async move {
            if let Err(e) = process(socket, server_start_time, db).await {
                error!("{:?}", e);
            }
        });
    }
}

pub async fn listen(port: u16) -> anyhow::Result<()> {
    tracing::trace!("listen");
    let server_start_time = Instant::now();
    let db: Arc<RwLock<Db>> = Default::default();

    let socket = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
    let future = accept_connenctions_loop(socket, server_start_time, db.clone());
    let v4 = spawn(future);
    let socket = SocketAddr::new(IpAddr::from_str("::").unwrap(), port);
    let future = accept_connenctions_loop(socket, server_start_time, db);
    let v6 = spawn(future);

    select! {
        result = v4 => {
            result??;
        }
        result = v6 => {
            result??;
        }
    }
    Ok(())
}
