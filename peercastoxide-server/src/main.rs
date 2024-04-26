mod pcp_server;
mod tracing_helper;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_helper::init();

    pcp_server::listen().await?;
    Ok(())
}
