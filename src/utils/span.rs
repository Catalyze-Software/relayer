use std::future::Future;

use tracing::{instrument::Instrumented, Instrument};

pub fn with_spans<T>(
    name: &str,
    fut: impl Future<Output = T>,
) -> Instrumented<impl Future<Output = T>> {
    fut.instrument(tracing::debug_span!("runner", name))
        .instrument(tracing::info_span!("runner", name))
        .instrument(tracing::error_span!("runner", name))
        .instrument(tracing::warn_span!("runner", name))
}
