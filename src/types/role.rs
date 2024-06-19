use std::str::FromStr;

use eyre::bail;

#[derive(Debug, Clone)]
pub enum Role {
    Owner = 100,
    Admin = 95,
    Moderator = 50,
    Member = 10,
}

impl FromStr for Role {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "moderator" => Ok(Self::Moderator),
            "member" => Ok(Self::Member),
            _ => {
                bail!("Invalid role: {}", s)
            }
        }
    }
}

impl Role {
    pub fn power_level(&self) -> u64 {
        self.clone() as u64
    }
}
