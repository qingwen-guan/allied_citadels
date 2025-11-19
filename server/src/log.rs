use std::path::Path;

use time::{UtcOffset, format_description};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{Layer, fmt};

pub fn init_log(name: &str) -> WorkerGuard {
  let stdout_layer = tracing_subscriber::fmt::layer()
    .with_target(false)
    .with_line_number(false)
    .with_file(false)
    .with_writer(std::io::stdout)
    .with_filter(LevelFilter::WARN);

  let file_appender = tracing_appender::rolling::never(Path::new("logs"), format!("{name}.log"));

  let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

  let timer_format = format_description::parse(
    "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second].[subsecond digits:6]",
  )
  .expect("Failed to parse time format description");

  // 2. Get the current local time offset
  let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC); // Fallback to UTC if local offset cannot be determined

  // 3. Create an OffsetTime timer with the local offset and format
  let timer = fmt::time::OffsetTime::new(local_offset, timer_format);

  let file_layer = tracing_subscriber::fmt::layer()
    .with_target(false)
    .with_writer(non_blocking)
    .with_timer(timer)
    .json()
    .with_filter(LevelFilter::INFO);

  let subscriber = tracing_subscriber::registry().with(stdout_layer).with(file_layer);

  tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

  _guard
}
