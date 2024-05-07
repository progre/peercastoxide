use std::{env, num::NonZeroU8, panic};

use time::format_description::well_known::{
    iso8601::{self, EncodedConfig},
    Iso8601,
};
use tracing::error;
use tracing_subscriber::{
    fmt::{
        self,
        format::{Compact, DefaultFields, Format},
        time::{FormatTime, LocalTime, SystemTime},
    },
    prelude::__tracing_subscriber_SubscriberExt,
    EnvFilter, Layer, Registry,
};

fn default_subscriber_builder() -> fmt::Layer<Registry, DefaultFields, Format<Compact>> {
    const WITH_FILE_PATH: bool = cfg!(debug_assertions);
    fmt::layer()
        .compact()
        .with_file(WITH_FILE_PATH)
        .with_line_number(WITH_FILE_PATH)
        .with_target(!WITH_FILE_PATH)
        .with_thread_ids(true)
}

type MyLayer<T> = fmt::Layer<Registry, DefaultFields, Format<Compact, T>>;

fn init_tracing<T: FormatTime + Send + Sync + 'static>(
    customize: fn(MyLayer<SystemTime>) -> MyLayer<T>,
) {
    let layer = customize(default_subscriber_builder());
    const DIRECTIVES: &str = if cfg!(debug_assertions) {
        concat!(env!("CARGO_CRATE_NAME"), "=trace,peercastoxide_lib=trace")
    } else {
        concat!(env!("CARGO_CRATE_NAME"), "=info,peercastoxide_lib=info")
    };
    let filter = EnvFilter::new(DIRECTIVES);
    let reg = tracing_subscriber::registry()
        .with(layer.with_filter(filter))
        .with(console_subscriber::ConsoleLayer::builder().spawn());
    tracing::subscriber::set_global_default(reg).unwrap();
    panic::set_hook(Box::new(|panic| error!("{}", panic)));
}

pub fn init() {
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
    const MY_CONFIG: EncodedConfig = iso8601::Config::DEFAULT
        .set_time_precision(iso8601::TimePrecision::Second {
            decimal_digits: NonZeroU8::new(6),
        })
        .encode();
    init_tracing(|layer| layer.with_timer(LocalTime::new(Iso8601::<MY_CONFIG>)));
}
