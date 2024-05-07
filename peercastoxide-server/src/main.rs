mod create_xml;

use std::num::NonZeroU16;

use clap::Parser;

mod pcp_server;
mod tracing_helper;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = NonZeroU16::new(80).unwrap())]
    http_port: NonZeroU16,
    #[arg(long, default_value_t = NonZeroU16::new(7144).unwrap())]
    pcp_port: NonZeroU16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_helper::init();

    let args = Args::parse();

    pcp_server::listen(args.http_port.get(), args.pcp_port.get()).await?;
    Ok(())
}
