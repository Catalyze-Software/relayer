use std::{fmt::Display, str::FromStr};

use proxy_types::models::history_event::HistoryEventKind;

#[derive(Debug, Clone)]
pub struct QueueKey {
    event_kind: HistoryEventKind,
}

impl From<HistoryEventKind> for QueueKey {
    fn from(event_kind: HistoryEventKind) -> Self {
        Self { event_kind }
    }
}

impl Display for QueueKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "queue_{}", self.event_kind)
    }
}

impl FromStr for QueueKey {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let event_kind = HistoryEventKind::from_str(s)
            .map_err(|e| eyre::eyre!("Failed to parse history event kind from string: {e}"))?;

        Ok(Self { event_kind })
    }
}
