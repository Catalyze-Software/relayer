use candid::CandidType;
use serde::Deserialize;

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct HistoryEvent {
    pub id: u64,
    pub payload: HistoryEventKind,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub enum HistoryEventKind {
    GroupMemberRoleChanged,
}

pub struct GroupMemberRoleChangedPayload {}
