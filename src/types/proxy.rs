use candid::CandidType;
use proxy_types::models::api_error::ApiError;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum HistoryPointResult {
    Ok(u64),
    Err(ApiError),
}
