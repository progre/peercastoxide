use std::{num::NonZeroU16, time::Duration};

use tokio::{net::TcpListener, spawn, time::timeout};

use super::proxy_for_get_channel::proxy_for_get_channel;

pub async fn listen_for(hostname_from_real_server: &str, tip_host: String) -> NonZeroU16 {
    let server = TcpListener::bind(&format!("{}:0", hostname_from_real_server))
        .await
        .unwrap();
    let port = server.local_addr().unwrap().port().try_into().unwrap();
    spawn(async move {
        let (client, _) = timeout(Duration::from_secs(10), server.accept())
            .await
            .unwrap()
            .unwrap();
        proxy_for_get_channel(client, tip_host).await.unwrap();
    });
    port
}
