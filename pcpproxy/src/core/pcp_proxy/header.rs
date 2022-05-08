use anyhow::Result;
use regex::Regex;
use tokio::net::tcp::ReadHalf;

fn find_channel_id_and_tip_host(line: &str) -> Option<(String, String)> {
    let regex =
        Regex::new(r"/[0-9A-Za-z]+/([0-9A-Fa-f]+)\?tip=([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+:[0-9]+)")
            .unwrap();
    let captures = &regex.captures(line)?;
    Some((captures[1].into(), captures[2].into()))
}

fn find_channel_id(line: &str) -> Option<String> {
    let regex = Regex::new(r"/channel/([0-9A-Fa-f]+)").unwrap();
    let captures = &regex.captures(line)?;
    Some(captures[1].into())
}

pub enum Header {
    GetChannel {
        channel_id: String,
    },
    GetWithTip {
        channel_id: String,
        tip_host: String,
    },
    Pcp,
    Http,
    Unknown,
    Empty,
}

pub async fn check_header(from: &mut ReadHalf<'_>) -> Result<Header> {
    let mut buf = [0u8; 1024];
    let n = from.peek(&mut buf).await?;
    if n == 0 {
        return Ok(Header::Empty);
    }
    if n > 4 && &buf[0..4] == b"pcp\n" {
        return Ok(Header::Pcp);
    }
    if n > 4 && &buf[0..4] == b"GET " {
        if let Some(idx) = buf[0..n].iter().position(|&x| x == b'\n') {
            let line = String::from_utf8(buf[0..idx].to_vec()).unwrap();
            if let Some(channel_id) = find_channel_id(&line) {
                return Ok(Header::GetChannel { channel_id });
            }
            if let Some((channel_id, tip_host)) = find_channel_id_and_tip_host(&line) {
                return Ok(Header::GetWithTip {
                    channel_id,
                    tip_host,
                });
            }
            return Ok(Header::Http);
        }
    }
    if n > 4 && &buf[0..4] == b"POST" {
        return Ok(Header::Http);
    }
    log::trace!(
        "Unknown: {:?}, {}",
        &buf[0..4],
        buf[0..4]
            .iter()
            .map(|&x| (x as char).to_string())
            .collect::<Vec<_>>()
            .join("")
    );
    Ok(Header::Unknown)
}
