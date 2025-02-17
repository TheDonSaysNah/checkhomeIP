use std::process;

use clap::Parser;
use time::{OffsetDateTime, UtcOffset};
use tokio::signal;
use tokio::{runtime::Builder, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{filter::LevelFilter, fmt::time::OffsetTime, EnvFilter};

mod check;
mod notify;

#[derive(Parser, Debug)]
#[clap(author="TheDonSaysNah", version=env!("CARGO_PKG_VERSION"), about="A small tool to monitor your home IP and alert you if it changes.", long_about = None)]
pub struct Args {
	#[arg(short='c', long, help="If true then notify the server with the client name and local IP")]
	client: Option<String>,

	#[arg(short='e', long, help="Notify by email",)]
	email: bool,

	#[arg(short='n', long, help="Notify by NTFY service: https://ntfy.sh")]
	ntfy: bool
}
fn main() {
	// Setup tracing with my preferred logging format
	// env::set_var("RUST_BACKTRACE", "full");
	let time_format = time::format_description::parse("[[[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]]").unwrap();
	let default = format!("{}={}", env!("CARGO_PKG_NAME"), if !cfg!(debug_assertions) { LevelFilter::INFO } else { LevelFilter::TRACE }).parse().unwrap();
	let offset = UtcOffset::local_offset_at(OffsetDateTime::UNIX_EPOCH).unwrap();
	let timer = OffsetTime::new(offset, time_format);

	let filter = EnvFilter::builder().with_default_directive(default).parse("").unwrap();
	let sub = tracing_subscriber::fmt().with_file(cfg!(debug_assertions)).with_line_number(cfg!(debug_assertions))
		.with_target(false).with_timer(timer).with_env_filter(filter).with_thread_ids(cfg!(debug_assertions)).with_thread_names(cfg!(debug_assertions)).finish();
	tracing::subscriber::set_global_default(sub).unwrap();

	// Load env vars. Unwrap is fine here because if vars are missing/incorrect then program can't run correctly anyway
	dotenv::from_path("checkip.env").unwrap();

	let runtime = Builder::new_multi_thread().thread_name("ip_runtime").enable_all().build().unwrap();
	let rt_cl = runtime.handle().clone();

	let token = CancellationToken::new();
	let token_cl = token.clone();

	let app: Args = Args::parse();
	if !app.email && !app.ntfy {
		tracing::error!("No notification channel has been specified! Use -h for more info.");
		process::exit(1);
	} else {
		let mut trues: Vec<&str> = vec![];
		if app.email { trues.push("email"); }
		if app.ntfy { trues.push("NTFY"); }
		tracing::info!("Using {} to send notifications.", trues.join(" and "))
	}

	runtime.spawn(async move {
		tokio::select! {
            _ = rt_cl.spawn(async move { check::CheckIP::init(app).await; }) => {}
            _ = token_cl.cancelled() => tracing::debug!("Token cancelled"),
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
