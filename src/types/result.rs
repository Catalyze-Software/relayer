use candid::{CandidType, Decode};
use catalyze_shared::api_error::ApiError;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum CanisterResult<T> {
    Ok(T),
    Err(ApiError),
}

impl<'de, 'a, T: CandidType + Deserialize<'de>> TryFrom<&'a [u8]> for CanisterResult<T>
where
    'a: 'de,
{
    type Error = eyre::Error;

    fn try_from(value: &'a [u8]) -> eyre::Result<Self> {
        match Decode!(value, CanisterResult<T>) {
            Ok(result) => Ok(result),
            Err(err) => Err(eyre::eyre!("{:#?}", err)),
        }
    }
}

impl<'de, T: CandidType + Deserialize<'de>> CanisterResult<T> {
    pub fn into_result(self) -> eyre::Result<T> {
        match self {
            CanisterResult::Ok(result) => Ok(result),
            CanisterResult::Err(err) => Err(eyre::eyre!("{:#?}", err)),
        }
    }
}
