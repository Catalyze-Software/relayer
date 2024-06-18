use std::fmt::Display;

use candid::Principal;

#[derive(Debug, Clone)]
pub struct MatrixUserID {
    pub principal: Principal,
    pub username: String,
    pub matrix_base_url: String,
}

impl MatrixUserID {
    pub fn new(principal: Principal, username: String, matrix_base_url: String) -> Self {
        Self {
            principal,
            username,
            matrix_base_url,
        }
    }
}

impl Display for MatrixUserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let principal = self.principal.to_string();
        let username = self.username.to_lowercase();
        let matrix_url = self
            .matrix_base_url
            .trim_start_matches("https://matrix.")
            .trim_start_matches("https://") // You ask me why? - I don't know
            .trim_end_matches('/');

        // Equal to how the front-end deterministically generates the user ID for the matrix
        write!(f, "@{principal}/{username}:{matrix_url}")
    }
}
