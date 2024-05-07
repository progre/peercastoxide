mod create_xml;

use std::num::NonZeroU16;

use clap::Parser;

mod pcp_server;
mod tracing_helper;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = NonZeroU16::new(7144).unwrap())]
    port: NonZeroU16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_helper::init();

    let args = Args::parse();

    pcp_server::listen(args.port.get()).await?;
    Ok(())
}
