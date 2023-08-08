use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq)]
pub struct JournalNew {
    pub title: String,
    pub text: String
}

#[derive(Serialize, Deserialize)]
pub struct DBInfo {
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: String
}