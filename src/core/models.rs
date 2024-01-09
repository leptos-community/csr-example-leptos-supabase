use chrono::{DateTime, Utc};
use leptos::{RwSignal, SignalGetUntracked};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Eq, Serialize, Deserialize, Hash)]
pub struct Company {
    pub name: RwSignal<String>,
    pub phone: RwSignal<String>,
    pub jobs: RwSignal<Vec<Job>>,
    pub date_added: DateTime<Utc>,
    #[serde(default = "default_status_for_serde")]
    pub status: RwSignal<Status>,
}

impl PartialEq for Company {
    fn eq(&self, other: &Self) -> bool {
        self.name.get_untracked() == other.name.get_untracked()
            && self.phone.get_untracked() == other.phone.get_untracked()
            && self.jobs.get_untracked() == other.jobs.get_untracked()
            && self.date_added == other.date_added
            && self.status.get_untracked() == other.status.get_untracked()
    }
}

fn default_status_for_serde() -> RwSignal<Status> {
    RwSignal::new(Status::Synced)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    pub qualification: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Failure {
    AccessTokenExpired,
    RefreshTokenExpired,
    Other,
}

pub enum RefreshTokenError {
    NetworkError,
    JsonParseError,
    RefreshTokenExpirationError,
    UnknownError,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub access_token: String,
    pub uuid: String,
    pub refresh_token: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Status {
    SyncingInsert,
    SyncingEdit,
    SyncingDelete,
    InsertFailed,
    EditFailed,
    DeleteFailed,
    Synced,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::SyncingInsert => write!(f, "Syncing Insert"),
            Status::SyncingEdit => write!(f, "Syncing Edit"),
            Status::SyncingDelete => write!(f, "Syncing Delete"),
            Status::InsertFailed => write!(f, "Insert failed"),
            Status::EditFailed => write!(f, "Edit failed"),
            Status::DeleteFailed => write!(f, "Delete failed"),
            Status::Synced => write!(f, "Synced"),
        }
    }
}

