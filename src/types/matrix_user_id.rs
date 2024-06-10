use std::fmt::Display;

use candid::Principal;

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
        let id = [
            self.principal.to_string(),
            self.username.clone(),
            self.matrix_base_url.clone(),
        ]
        .join(":");

        // Equal to how the front-end deterministically generates the user ID for the matrix
        write!(f, "@{id}")
    }
}
