use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum HistoryPointResult {
    Ok(u64),
    Err(ApiError),
}

#[derive(Debug, CandidType, Deserialize)]
pub enum ApiErrorType {
    Duplicate,
    SerializeError,
    DeserializeError,
    NotFound,
    ValidationError(Vec<ValidationResponse>),
    Unsupported,
    Unauthorized,
    Unexpected,
    NotImplemented,
    BadRequest,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct ValidationResponse {
    pub field: String,
    pub message: String,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct ApiError {
    pub tag: Option<String>,
    pub info: Option<Vec<String>>,
    pub method_name: Option<String>,
    pub message: Option<String>,
    pub timestamp: u64,
    pub error_type: ApiErrorType,
}
