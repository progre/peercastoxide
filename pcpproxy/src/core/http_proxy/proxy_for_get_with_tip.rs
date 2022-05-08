use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::join;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::spawn;

use crate::core::pcp_proxy::pipe::big_vec;
use crate::core::pcp_proxy::pipe::pipe_raw;
use crate::features::output::ndjson::NDJson;

struct DropImpl<F>
where
    F: FnMut(),
{
    drop: F,
}
impl<F> Drop for DropImpl<F>
where
    F: FnMut(),
{
    fn drop(&mut self) {
        (self.drop)();
    }
}

async fn pipe_for_get_with_tip(
    mut incoming: OwnedReadHalf,
    mut outgoing: OwnedWriteHalf,
    tip_host: &str,
    host_from_real_server: &str,
    output: NDJson,
) -> Result<()> {
    let mut buf = big_vec(1024 * 1024);
    loop {
        let n = incoming.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        output.output_raw(&String::from_utf8_lossy(&buf[0..n]));
        // GET /{path}/{id}?tip={host} HTTP/1.1 id をキーに host を保存
        if let Some(idx) = buf[0..n].iter().position(|&x| x == b'\n') {
            let line = String::from_utf8(buf[0..idx].to_vec()).unwrap();
            let replaced_line = line.replace(tip_host, host_from_real_server);
            outgoing.write_all(replaced_line.as_bytes()).await?;
            outgoing.write_all(&buf[idx..n]).await?;
            continue;
        }
        outgoing.write_all(&buf[0..n]).await?;
    }
}

async fn proxy_for_get_with_tip_internal(
    client: TcpStream,
    host_from_real_server: String,
    server_host: &str,
    tip_host: String,
) -> Result<()> {
    let client_addr = client.peer_addr()?.to_string();
    let (client_incoming, client_outgoing) = client.into_split();
    let server = TcpStream::connect(server_host).await?;
    let (server_incoming, server_outgoing) = server.into_split();

    let upload_handle = {
        let client_addr = client_addr.clone();
        let server_host_string = server_host.to_owned();
        spawn(async move {
            pipe_for_get_with_tip(
                client_incoming,
                server_outgoing,
                &tip_host,
                &host_from_real_server,
                NDJson::upload(client_addr, server_host_string),
            )
            .await
        })
    };
    let download_handle = {
        let server_host_string = server_host.to_owned();
        spawn(async move {
            pipe_raw(
                server_incoming,
                client_outgoing,
                NDJson::download(client_addr, server_host_string),
            )
            .await
        })
    };
    let (upload_result, download_result) = join!(upload_handle, download_handle);
    upload_result??;
    download_result??;
    Ok(())
}

pub async fn proxy_for_get_with_tip(
    client: TcpStream,
    host_from_real_server: String,
    server_host: &str,
    channel_id: String,
    tip_host: String,
    channel_id_host_pair: &std::sync::Mutex<Vec<(String, String)>>,
) -> Result<()> {
    let pair = (channel_id.to_uppercase(), tip_host.clone());
    channel_id_host_pair.lock().unwrap().push(pair.clone());
    let result =
        proxy_for_get_with_tip_internal(client, host_from_real_server, server_host, tip_host).await;
    let mut vec = channel_id_host_pair.lock().unwrap();
    let idx = vec.iter().position(|x| x == &pair).unwrap();
    vec.remove(idx);
    result
}
