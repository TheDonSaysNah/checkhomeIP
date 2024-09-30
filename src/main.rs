use std::{env, process};

use dotenv::dotenv;
use time::{UtcOffset, OffsetDateTime};
use tokio::signal;
use tokio::{runtime::Builder, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt::time::OffsetTime};

mod check;
mod mail;

fn main() {
    // Setup tracing with my preferred logging format
    // env::set_var("RUST_BACKTRACE", "full");
    let time_format = time::format_description::parse("[[[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]]").unwrap();
    let default = format!("{}={}", env!("CARGO_PKG_NAME"), if !cfg!(debug_assertions) { LevelFilter::INFO } else { LevelFilter::TRACE }).parse().unwrap();
    let offset = UtcOffset::local_offset_at(OffsetDateTime::UNIX_EPOCH).unwrap();
    let timer = OffsetTime::new(offset, time_format);

    let filter = EnvFilter::builder().with_default_directive(default).parse("").unwrap();
    let sub = tracing_subscriber::fmt().with_file(true).with_line_number(true)
        .with_target(false).with_timer(timer).with_env_filter(filter).with_thread_ids(cfg!(debug_assertions)).with_thread_names(cfg!(debug_assertions)).finish();
    tracing::subscriber::set_global_default(sub).unwrap();

    // Load env vars. Unwrap is fine here because if vars are missing/incorrect then program can't run correctly anyway
    tracing::info!("Using {:?}", dotenv().unwrap());

    let runtime = Builder::new_multi_thread().thread_name("ip_runtime").enable_all().build().unwrap();
    let rt_cl = runtime.handle().clone();

    let token = CancellationToken::new();
    let token_cl = token.clone();

    runtime.spawn(async move {
        tokio::select! {
            _ = rt_cl.spawn(async move { check::CheckIP::init().await; }) => {}
            _ = token_cl.cancelled() => tracing::warn!("Token cancelled"),
        }
    });


    runtime.block_on(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                tracing::warn!("Received signal");
                token.cancel();
            }
            Err(e) => {
                tracing::error!("Failed to listen for CTRL+C signal: {e}");
                process::exit(1);
            }
        }
    });

    tracing::warn!("Shutting down main runtime...");
    runtime.shutdown_timeout(Duration::from_secs(5));
}
