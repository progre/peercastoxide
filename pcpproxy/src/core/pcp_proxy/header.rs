use anyhow::Result;
use tokio::net::tcp::ReadHalf;

pub enum Header {
    Pcp,
    Http,
    Unknown,
    Empty,
}

pub async fn check_header(from: &mut ReadHalf<'_>) -> Result<Header> {
    let mut buf = [0u8; 4];
    let n = from.peek(&mut buf).await?;
    Ok(match &buf {
        b"pcp\n" => Header::Pcp,
        b"GET " => Header::Http,
        b"POST" => Header::Http,
        _ if n == 0 => Header::Empty,
        _ => {
            log::trace!(
                "Unknown: {:?}, {}",
                &buf,
                buf.iter()
                    .map(|&x| (x as char).to_string())
                    .collect::<Vec<_>>()
                    .join("")
            );
            Header::Unknown
        }
    })
}
