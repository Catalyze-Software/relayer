use std::str::FromStr;

use eyre::bail;

#[derive(Debug, Clone)]
pub enum Role {
    Owner,
    Admin,
    Moderator,
    Member,
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
        match self {
            Self::Owner => 100,
            Self::Admin => 95,
            Self::Moderator => 50,
            Self::Member => 10,
        }
    }
}
