use candid::CandidType;
use proxy_types::models::{api_error::ApiError, history_event::HistoryEventEntry};
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum HistoryEventResult {
    Ok(Vec<HistoryEventEntry>),
    Err(ApiError),
}
