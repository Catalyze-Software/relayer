use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MatrixAuthTokenResponse {
    pub token: String,
}
